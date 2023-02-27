use crate::middleware_specs;

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
    fmt::Debug,
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
        SelectAll
    },
    future::BoxFuture
};
pub struct PoolConnection<T: ConnectionHandler> {
    conn: Pin<Box<Connection<T>>>,
    handler: T,
    event: mpsc::Sender<ConnectionHandlerInEvent<T>>
}

pub enum PoolEvent<T: ConnectionHandler> {
    ConnectionEstablished(PoolConnection<T>),
    ConnectionClosed(PoolConnection<T>),
    ConnectionEvent(PoolConnection<T>),
}


impl<T: ConnectionHandler> PoolEvent<T> {
    fn notify_event<'a>(
        &'a self, 
        mut events: mpsc::Sender<&'a Self>
    ) -> Result<(), mpsc::TrySendError<&'a Self>>
    {
        match self {
            PoolEvent::ConnectionEstablished(conn) => events.try_send(self),
            PoolEvent::ConnectionClosed(conn) => events.try_send(self),
            PoolEvent::ConnectionEvent(conn) => events.try_send(self)
        }
    }
}

pub struct PoolConfig {
    pub task_event_buffer_size: usize
}

pub struct Pool<T: ConnectionHandler + Debug, U> {
    pool_id: usize,
    counters: PoolConnectionCounters,
    pending: HashMap<ConnectionId, PendingConnection<T>>,
    established: HashMap<ConnectionId, EstablishedConnection<T>>,
    // This spawner is for connections buonded to T: Connectionhandler
    pub local_spawns: FuturesUnordered<Pin<Box<dyn Future<Output = T> + Send>>>,
    // These streams are for the incoming data streams of type U
    pub local_streams: SelectAll<Pin<Box<dyn futures::Stream<Item = U>>>>,
    executor: Option<Box<dyn Executor<T> + Send>>,
    pending_connection_events_tx: mpsc::Sender<ConnectionHandlerOutEvent<T>>,
    pending_connection_events_rx: mpsc::Receiver<ConnectionHandlerOutEvent<T>>,
    established_connection_events_tx: mpsc::Sender<ConnectionHandlerOutEvent<T>>,
    established_connection_events_rx: mpsc::Receiver<ConnectionHandlerOutEvent<T>>,
}

impl<T, U> Pool<T, U> 
where
T: ConnectionHandler + Debug,
U: Send + 'static
{
    pub fn new(pool_id: usize, config: PoolConfig, limits: PoolConnectionLimits) -> Pool<T, U> {
        let (pending_connection_events_tx, pending_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        let (established_connection_events_tx, established_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        Pool {
            pool_id,
            counters: PoolConnectionCounters::default() ,
            pending: Default::default(),
            established: Default::default(),
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
    pub fn spawn(&mut self, task: BoxFuture<'static, T>) {
        if let Some(executor) = &mut self.executor {
            // If there's an executor defined for this Pool then we use it.
            executor.exec(task);
        } else {
            // Otherwise we push the task to a FuturesUnordered collection.
            self.local_spawns.push(task);
        }
    }
    pub fn inject_connection(&self, conn: impl Future<Output = T>) {
        ()
    }

    pub fn collect_streams(&mut self, stream: Pin<Box<dyn futures::Stream<Item = U >>>) {
        self.local_streams.push(stream);
    }

    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<PoolEvent<T>> {
        match self.established_connection_events_rx.poll_next_unpin(cx) {
            Poll::Pending => {},
            Poll::Ready(None) => println!("Pool is None"),
            Poll::Ready(Some(a)) => { println!("Received from connection handler: {:?}", a); }
        };
        Poll::Pending
    }
}

pub struct PendingConnection<T: ConnectionHandler> (PoolConnection<T>);

pub struct EstablishedConnection<T: ConnectionHandler> (PoolConnection<T>);

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