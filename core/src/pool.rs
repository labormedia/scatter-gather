use crate::middleware_specs::{ServerConfig};

/// Pool Connection is initialized with the handler defined 
/// for the specific Connection::source_type::handler.
/// 
/// 
use super::{
    Executor,
    connection::{
        Connection,
        ConnectionId,
        ConnectionHandler,
        ConnectionHandlerInEvent,
        ConnectionHandlerOutEvent
    },
};
use std::{
    collections::HashMap,
    pin::Pin, 
    any::type_name,
    fmt::Debug,
    sync::{
        Arc
    },
    hash::{
        Hash,
        Hasher
    },
    ops::AddAssign
};
use futures::{
    channel::mpsc,
    StreamExt,
    task::{
        Poll,
        Context
    },
    Future,
    stream::{
        FuturesUnordered,
        SelectAll, Next
    }, SinkExt,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct PoolConnection<T: for <'a> ConnectionHandler<'a>> {
    // conn: Pin<Box<Connection>>,
    id: ConnectionId,
    state: Arc<Mutex<T>>
}

impl<T: for<'a> ConnectionHandler<'a>> PartialEq for PoolConnection<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: for <'a> ConnectionHandler<'a>> Eq for PoolConnection<T> {}

impl<T: for<'a> ConnectionHandler<'a>> Hash for PoolConnection<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub enum PoolEvent<T: for<'a> ConnectionHandler<'a>> {
    ConnectionEstablished(PoolConnection<T>),
    ConnectionClosed(PoolConnection<T>),
    ConnectionEvent(PoolConnection<T>),
    Custom
}

impl<T: for <'a> ConnectionHandler<'a>> Debug for PoolEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Default")
    }
}

impl<'b, T: for <'a> ConnectionHandler<'a>> PoolEvent<T> {
    fn notify_event<'a>(
        &'a self, 
        mut events: mpsc::Sender<&'a Self>
    ) -> Result<(), mpsc::TrySendError<&Self>>
    {
        match self {
            PoolEvent::ConnectionEstablished(conn) => events.try_send(self),
            PoolEvent::ConnectionClosed(conn) => events.try_send(self),
            PoolEvent::ConnectionEvent(conn) => events.try_send(self),
            PoolEvent::Custom => {
                #[cfg(debug_assertions)]
                println!("Custom assertion for PoolEvent : {:?}", self);
                events.try_send(&PoolEvent::Custom)
            }
        }
    }
}

pub struct PoolConfig {
    pub task_event_buffer_size: usize
}

pub struct Pool<T: for <'a> ConnectionHandler<'a> + Debug, U> {
    pool_id: usize,
    counters: PoolConnectionCounters,
    pending: HashMap<usize, PendingConnection<T>>,
    established: HashMap<usize, EstablishedConnection<T>>,
    next_connection_id: ConnectionId,
    // This spawner is for connections bounded to T: Connectionhandler
    pub local_spawns: FuturesUnordered<Pin<Box<dyn Future<Output = T> + Send>>>,
    // These streams are for the incoming data streams of type U
    pub local_streams: SelectAll<Pin<Box<dyn futures::Stream<Item = U>>>>,
    executor: Option<Box<dyn Executor<T> + Send>>,
    pending_connection_events_tx: mpsc::Sender<PendingConnection<T>>,
    pending_connection_events_rx: mpsc::Receiver<PendingConnection<T>>,
    established_connection_events_tx: mpsc::Sender<EstablishedConnection<T>>,
    established_connection_events_rx: mpsc::Receiver<EstablishedConnection<T>>,
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

impl<'b, T, U> Pool<T, U>
where
T: for <'a> ConnectionHandler<'a> + Debug + Send + Sync,
// T: for <'a> ConnectionHandler<'a> + Debug + Sync + for <'a> ConnectionHandler<'a, OutEvent = T::<Self, 'a>>,
U: Send + 'static + std::fmt::Debug
{
    pub fn new<'a>(pool_id: usize, config: PoolConfig, limits: PoolConnectionLimits) -> Pool<T, U> {
        let (pending_connection_events_tx, pending_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        let (established_connection_events_tx, established_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        Pool {
            pool_id,
            counters: PoolConnectionCounters::default() ,
            pending: HashMap::new(),
            established: HashMap::new(),
            next_connection_id: ConnectionId::new(0),
            local_spawns: FuturesUnordered::new(),
            local_streams: SelectAll::new(),
            executor: None,
            pending_connection_events_tx,
            pending_connection_events_rx,
            established_connection_events_tx,
            established_connection_events_rx,
        }
    }
    pub fn with_executor(mut self, e: Box<dyn Executor<T> + Send>) -> Self {
        self.executor = Some(e);
        self
    }
    pub fn spawn(&mut self, task: Pin<Box<dyn Future<Output = T> + Send>>) {
        if let Some(executor) = &self.executor {
            // If there's an executor defined for this Pool then we use it.
            executor.exec(task);
        } else {
            // Otherwise we push the task to a FuturesUnordered collection.
            self.local_spawns.push(task);
        }
    }
    pub fn inject_connection(&mut self, conn: impl Future<Output = T> + Send + 'static) {
        self.spawn(Box::pin(conn));
        self.counters.pending_incoming = self.counters.pending_incoming + 1;
    }

    pub fn collect_streams(&mut self, stream: Pin<Box<dyn futures::Stream<Item = U >>>) {
        self.local_streams.push(stream);
    }

    pub async fn connect(self) -> Poll<Arc<Mutex<T>>> {
        match self.poll(&mut Context::from_waker(futures::task::noop_waker_ref())).await
        {
            Poll::Ready(event) => { 
                #[cfg(debug_assertions)]
                println!("Poll Ready... : {event:?}");
                return match event {
                    ConnectionHandlerOutEvent::ConnectionEvent(pool_conn) => {
                        let a = pool_conn.state;
                        return Poll::Ready(a);
                    }
                    _ => {
                        Poll::Pending
                    }
                }

            }
            Poll::Pending => { 
                #[cfg(debug_assertions)]
                println!("Poll pending...");
                Poll::Pending
            },
        } 
    }

    pub async fn intercept_stream(&mut self) {
        loop {
            match self.local_streams.next().await {
                Some(a) => {
                    #[cfg(debug_assertions)]
                    println!("Received: {:?}", a);
                },
                _ => {}

            }
        }
           
    }

    pub fn next_connection_id(&mut self) -> ConnectionId {
        let res = self.next_connection_id;
        self.next_connection_id = self.next_connection_id + 1;
        res
    }

    pub fn next(&mut self) -> Next<SelectAll<Pin<Box<dyn futures::Stream<Item = U>>>>> {
        self.local_streams.next()
    }

    pub async fn poll(mut self, cx: &mut Context<'_>) -> Poll<ConnectionHandlerOutEvent<PoolConnection<T>>>
    {
        #[cfg(debug_assertions)]
        println!("Entering Pool poll.");
        return loop {
            while let Some(s) = self.local_spawns.next().await {
                let pool_connection = PoolConnection {
                    id: self.next_connection_id(),
                    state: Arc::new(Mutex::new(s))
                };
                // let pool = self.established.entry(pool_connection.id).or_insert(EstablishedConnection(&pool_connection));
                self.established_connection_events_tx.send(EstablishedConnection(pool_connection) ).await.expect("Could not send established connection.");
                self.counters.established_incoming = self.counters.established_incoming + 1;
                self.counters.pending_incoming = self.counters.pending_incoming - 1;             
            };
    
            if let Some(EstablishedConnection(t)) = self.established_connection_events_rx.next().await {
                
                // self.established.insert(t.id, EstablishedConnection(&t));
                break Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(t))
            } else {
                break Poll::Pending
            };
        };

    }
}

pub struct PendingConnection<T: for <'a> ConnectionHandler<'a>> (PoolConnection<T>);

pub struct EstablishedConnection<T: for<'a> ConnectionHandler<'a>> (PoolConnection<T>);

#[derive(Debug, Clone, Default)]
pub struct PoolConnectionCounters {
    /// The effective connection limits.
    limits: PoolConnectionLimits,
    /// The current number of incoming connections.
    pending_incoming: u32,
    /// The current number of outgoing connections.
    pending_outgoing: u32,
    /// The current number of established inbound connections.
    established_incoming: u32,
    /// The current number of established outbound connections.
    established_outgoing: u32,
}

#[derive(Debug, Clone)]
pub struct PoolConnectionLimits {
    max_pending_incoming: Option<u32>,
    max_pending_outgoing: Option<u32>,
    max_established_incoming: Option<u32>,
    max_established_outgoing: Option<u32>,
    max_established_per_peer: Option<u32>,
    max_established_total: Option<u32>,
}

impl Default for PoolConnectionLimits {
    fn default() -> Self {
        Self {
            max_pending_incoming: Some(2),
            max_pending_outgoing: Some(2),
            max_established_incoming: Some(4),
            max_established_outgoing: Some(4),
            max_established_per_peer: Some(4),
            max_established_total: Some(4),
        }
    }
}