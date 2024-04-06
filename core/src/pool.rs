/// Pool Connection is initialized with the handler defined 
/// for the specific Connection::source_type::handler.
/// 
/// 
use super::{
    Executor,
    connection::{
        ConnectionId,
        ConnectionHandler,
        ConnectionHandlerOutEvent
    },
};
use std::{
    collections::HashMap,
    pin::Pin, 
    fmt::Debug,
    sync::Arc,
    hash::{
        Hash,
        Hasher
    },
    error::Error,
};
use core::{
    ops::Add,
    task::{
        Poll,
        Context
    },
};
use futures::{
    channel::mpsc,
    StreamExt,
    Future,
    stream::{
        FuturesUnordered,
        SelectAll, Next
    }, SinkExt,
    TryFutureExt,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct PoolConnection<T: for <'a> ConnectionHandler<'a>, Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> {
    // conn: Pin<Box<Connection>>,
    pub id: ConnectionId<Id>,
    pub state: Arc<Mutex<T>>
}

impl<T, Id> PartialEq for PoolConnection<T, Id>
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T, Id> Eq for PoolConnection<T, Id> 
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{}

impl<T, Id> Hash for PoolConnection<T, Id> 
where
T: for<'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub enum PoolEvent<T: for<'a> ConnectionHandler<'a>, Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> {
    ConnectionEstablished(PoolConnection<T, Id>),
    ConnectionClosed(PoolConnection<T, Id>),
    ConnectionEvent(PoolConnection<T, Id>),
    Custom
}

impl<T, Id> Debug for PoolEvent<T, Id> 
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Default")
    }
}

impl<T, Id> PoolEvent<T, Id> 
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    fn notify_event<'a>(
        &'a self, 
        mut events: mpsc::Sender<&'a Self>
    ) -> Result<(), mpsc::TrySendError<&Self>>
    {
        match self {
            PoolEvent::ConnectionEstablished(_conn) => events.try_send(self),
            PoolEvent::ConnectionClosed(_conn) => events.try_send(self),
            PoolEvent::ConnectionEvent(_conn) => events.try_send(self),
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

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            task_event_buffer_size: 1_usize,
        }
    }
}

pub struct Pool<T, InBound, Id>
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    _pool_id: Id,
    counters: PoolConnectionCounters,
    last_connection_id: ConnectionId<Id>,
    pending: HashMap<Id, Pin<Box<dyn Future<Output = PendingConnection<T, Id>> + Send>> >,
    _established: HashMap<Id, EstablishedConnection<T, Id>>,
    // This spawner is for connections bounded to T: Connectionhandler
    pub local_spawns: FuturesUnordered<Pin<Box<dyn Future<Output = T> + Send>>>,
    // These streams are for the incoming data streams of type InBound
    pub local_streams: SelectAll<Pin<Box<dyn futures::Stream<Item = InBound>>>>,
    executor: Option<Box<dyn Executor<T> + Send>>,
    pending_connection_events_tx: mpsc::Sender<PendingConnection<T, Id>>,
    pending_connection_events_rx: mpsc::Receiver<PendingConnection<T, Id>>,
    established_connection_events_tx: mpsc::Sender<EstablishedConnection<T, Id>>,
    established_connection_events_rx: mpsc::Receiver<EstablishedConnection<T, Id>>,
}

impl<'b, T, InBound, Id> Pool<T, InBound, Id>
where
T: for <'a> ConnectionHandler<'a> + Debug + Send + Sync,
// T: for <'a> ConnectionHandler<'a> + Debug + Sync + for <'a> ConnectionHandler<'a, OutEvent = T::<Self, 'a>>,
InBound: Send + 'static + Debug,
Id: Default + From<bool> + Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,
{
    pub fn new(_pool_id: Id, config: PoolConfig, _limits: PoolConnectionLimits) -> Pool<T, InBound, Id> {
        let (pending_connection_events_tx, pending_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        let (established_connection_events_tx, established_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        Pool {
            _pool_id,
            counters: PoolConnectionCounters::default(),
            last_connection_id: ConnectionId::default(),
            pending: HashMap::new(),
            _established: HashMap::new(),
            local_spawns: FuturesUnordered::new(),
            local_streams: SelectAll::new(),
            executor: None,
            pending_connection_events_tx,
            pending_connection_events_rx,
            established_connection_events_tx,
            established_connection_events_rx,
        }
    }
    pub fn with_executor(&mut self, e: Box<dyn Executor<T> + Send>) {
        self.executor = Some(e);
    }
    pub fn spawn(&mut self, task: Pin<Box<dyn Future<Output = T> + Send>>) {
        if let Some(executor) = &self.executor {
            // If there's an executor defined for this Pool then we use it.
            #[cfg(debug_assertions)]
            println!("Executing task.");
            executor.exec(task);
        } else {
            // Otherwise we push the task to a FuturesUnordered collection.
            self.local_spawns.push(task);
        }
    }
    pub fn inject_connection(&mut self, conn: impl Future<Output = T> + Send + 'static) {
        self.spawn(Box::pin(conn));
        self.counters.pending_incoming += 1;
        self.last_connection_id.incr();
    }
    pub fn eject_connection(&mut self) -> Next<mpsc::Receiver<EstablishedConnection<T, Id>>> {
        self.established_connection_events_rx.next()
    }

    pub fn collect_streams(&mut self, stream: Pin<Box<dyn futures::Stream<Item = InBound >>>) {
        self.local_streams.push(stream);
    }

    pub async fn connect(&mut self) -> Poll<PoolConnection<T, Id>> {
        match self.poll(&mut Context::from_waker(futures::task::noop_waker_ref())).await
        {
            Poll::Ready(event) => { 
                #[cfg(debug_assertions)]
                println!("Poll Ready... : {event:?}");
                match event {
                    ConnectionHandlerOutEvent::ConnectionEstablished(pool_conn) => {
                        // let a = pool_conn.state;
                        Poll::Ready(pool_conn)
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
            if let Some(a) = self.local_streams.next().await {
                #[cfg(debug_assertions)]
                println!("Received: {:?}", a);
            } else {}
        }
           
    }

    pub fn next(&mut self) -> Next<SelectAll<Pin<Box<dyn futures::Stream<Item = InBound>>>>> {
        self.local_streams.next()
    }

    pub async fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<ConnectionHandlerOutEvent<PoolConnection<T, Id>>>
    {
        loop {
            if let Some(conn) = self.local_spawns.next().await {
                self.pending_connection_events_tx.send(PendingConnection(PoolConnection{
                    id: ConnectionId(self._pool_id), 
                    state: Arc::new(Mutex::new(conn)),
            }));
            };
            break;
        };
        Poll::Pending
        // #[cfg(debug_assertions)]
        // println!("Entering Pool poll.");
        // while let Some(s) = self.local_spawns.next().await {
        //     let pool_connection = PoolConnection {
        //         id: self.next_connection_id(),
        //         state: Arc::new(Mutex::new(s))
        //     };
        //     #[cfg(debug_assertions)]
        //     println!("Pool Connection {:?}", pool_connection);
        //     self.established_connection_events_tx.send(EstablishedConnection(pool_connection) ).await.expect("Could not send established connection.");
        //     self.counters.established_incoming += 1;
        //     self.counters.pending_incoming -= 1;             
        // };

        // if let Some(EstablishedConnection(c)) = self.eject_connection().await {
        //     #[cfg(debug_assertions)]
        //     println!("Connection Established.");
        //     Poll::Ready(ConnectionHandlerOutEvent::ConnectionEstablished(c))
        // } else {
        //     #[cfg(debug_assertions)]
        //     println!("Connection Pending.");
        //     Poll::Pending
        // }
    }
}

pub struct PendingConnection<T, Id> (PoolConnection<T, Id>)
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>;

pub struct EstablishedConnection<T, Id> (PoolConnection<T, Id>)
where
T: for <'a> ConnectionHandler<'a>,
Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>,;

#[derive(Debug, Clone, Default)]
pub struct PoolConnectionCounters {
    /// The effective connection limits.
    _limits: PoolConnectionLimits,
    /// The current number of incoming connections.
    pending_incoming: u32,
    /// The current number of outgoing connections.
    _pending_outgoing: u32,
    /// The current number of established inbound connections.
    established_incoming: u32,
    /// The current number of established outbound connections.
    _established_outgoing: u32,
}

#[derive(Debug, Clone)]
pub struct PoolConnectionLimits {
    _max_pending_incoming: Option<u32>,
    _max_pending_outgoing: Option<u32>,
    _max_established_incoming: Option<u32>,
    _max_established_outgoing: Option<u32>,
    _max_established_per_peer: Option<u32>,
    _max_established_total: Option<u32>,
}

impl Default for PoolConnectionLimits {
    fn default() -> Self {
        Self {
            _max_pending_incoming: Some(2),
            _max_pending_outgoing: Some(2),
            _max_established_incoming: Some(4),
            _max_established_outgoing: Some(4),
            _max_established_per_peer: Some(4),
            _max_established_total: Some(4),
        }
    }
}