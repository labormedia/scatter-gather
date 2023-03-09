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
    }
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
    id: usize,
    state: Arc<Mutex<T>>
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
    pending: HashMap<Connection, PendingConnection<T>>,
    established: HashMap<Connection, EstablishedConnection<T>>,
    // This spawner is for connections buonded to T: Connectionhandler
    pub local_spawns: FuturesUnordered<Pin<Box<dyn Future<Output = T> + Send>>>,
    // These streams are for the incoming data streams of type U
    pub local_streams: SelectAll<Pin<Box<dyn futures::Stream<Item = U>>>>,
    executor: Option<Box<dyn Executor<T> + Send>>,
    pending_connection_events_tx: mpsc::Sender<PoolConnection<T>>,
    pending_connection_events_rx: mpsc::Receiver<PoolConnection<T>>,
    established_connection_events_tx: mpsc::Sender<PoolConnection<T>>,
    established_connection_events_rx: mpsc::Receiver<PoolConnection<T>>,
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

    pub fn next(&mut self) -> Next<SelectAll<Pin<Box<dyn futures::Stream<Item = U>>>>> {
        self.local_streams.next()
    }

    pub async fn poll(mut self, cx: &mut Context<'_>) -> Poll<ConnectionHandlerOutEvent<PoolConnection<T>>>
    {
        println!("Entering Pool poll.");
        while let Some(s) = self.local_spawns.next().await {
            println!("Debug 1");
            let pool_connection = PoolConnection {
                id: 0_usize,
                state: Arc::new(Mutex::new(s))
            };
            self.pending_connection_events_tx.send(pool_connection).await.expect("Could not send pending connection.");                
        };

        if let Some(t) = self.pending_connection_events_rx.next().await {
            println!("Debug 2");
            // if let Ok(state) = t.state.lock() {
            //     state.as_any().downcast_ref::<T>().unwrap();
            //     println!("Received type: {:?}", state);
            // };
            return Poll::Ready(ConnectionHandlerOutEvent::ConnectionEvent(t))
        };
        println!("Dispatching Pool poll.");
        Poll::Pending
    }
}

pub struct PendingConnection<T: for <'a> ConnectionHandler<'a>> (PoolConnection<T>);

pub struct EstablishedConnection<T: for<'a> ConnectionHandler<'a>> (PoolConnection<T>);

#[derive(Debug, Clone)]
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

impl Default for PoolConnectionCounters {
    fn default() -> Self {
        Self {
            limits: PoolConnectionLimits::default(),
            pending_incoming: 4,
            pending_outgoing: 4,
            established_incoming: 4,
            established_outgoing: 4
        }
    }

}

#[derive(Debug, Clone, Default)]
pub struct PoolConnectionLimits {
    max_pending_incoming: Option<u32>,
    max_pending_outgoing: Option<u32>,
    max_established_incoming: Option<u32>,
    max_established_outgoing: Option<u32>,
    max_established_per_peer: Option<u32>,
    max_established_total: Option<u32>,
}