use futures_util::{
    StreamExt,
    future::BoxFuture,
    stream::FuturesUnordered,
    stream::SelectAll
};
use tokio::io::{AsyncReadExt};
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use futures::{
    channel::mpsc,
    SinkExt
};

pub mod middleware_specs;
pub mod connection;

use self::connection::*;
use self::middleware_specs::Interceptor;

pub trait Executor<T: Send> {
    fn exec(&self, future: Pin<Box<dyn Future<Output = T> + Send>>);
}

impl<T, F: Fn(Pin<Box<dyn Future<Output = T> + Send>>)> Executor<T> for F 
where
T: Send
{
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        self(f)
    }
}

/// Pool Connection is initialized with the handler defined 
/// for the specific Connection::source_type::handler.
pub struct PoolConnection<THandler: Interceptor> {
    conn: Pin<Box<Connection<THandler>>>,
    handler: THandler
}

pub enum PoolEvent<THandler: Interceptor> {
    ConnectionEstablished(PoolConnection<THandler>),
    ConnectionClosed(PoolConnection<THandler>),
    ConnectionEvent(PoolConnection<THandler>),
}


impl<THandler: Interceptor> PoolEvent<THandler> 
where
    THandler: ConnectionHandler
{
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

pub struct Pool<T, THandler: Interceptor, TError> {
    local_id: usize,
    counters: ConnectionCounters,
    pending: HashMap<ConnectionId, PendingConnection<THandler>>,
    established: HashMap<ConnectionId, EstablishedConnection<THandler>>,
    pub local_spawns: FuturesUnordered<Pin<Box<dyn Future<Output = T> + Send>>>,
    pub local_streams: SelectAll<Pin<Box<dyn futures::Stream<Item = T>>>>,
    executor: Option<Box<dyn Executor<T> + Send>>,
    pending_connection_events_tx: mpsc::Sender<ConnectionHandlerEvent<THandler, TError>>,
    pending_connection_events_rx: mpsc::Receiver<ConnectionHandlerEvent<THandler, TError>>,
    established_connection_events_tx: mpsc::Sender<ConnectionHandlerEvent<THandler, TError>>,
    established_connection_events_rx: mpsc::Receiver<ConnectionHandlerEvent<THandler, TError>>,
}

impl<T, THandler: Interceptor, TError> Pool<T, THandler, TError> 
where
T: Send + 'static
{
    pub fn new(local_id: usize, config: PoolConfig, limits: ConnectionLimits) -> Pool<T, THandler, TError> {
        let (pending_connection_events_tx, pending_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        let (established_connection_events_tx, established_connection_events_rx) =
            mpsc::channel(config.task_event_buffer_size);
        Pool {
            local_id,
            counters: ConnectionCounters::default() ,
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
    fn spawn(&mut self, task: BoxFuture<'static, T>) {
        if let Some(executor) = &mut self.executor {
            // If there's an executor defined for this Pool then we use it.
            executor.exec(task);
        } else {
            // Otherwise we push the task to a FuturesUnordered collection.
            self.local_spawns.push(task);
        }
    }

    pub fn collect_streams(&mut self, stream: Pin<Box<dyn futures::Stream<Item = T>>>) {
        self.local_streams.push(stream);
    }


}

pub struct PendingConnection<THandler: Interceptor>(PoolConnection<THandler>);

pub struct EstablishedConnection<THandler: Interceptor>(PoolConnection<THandler>);

#[derive(Debug, Clone)]
pub struct ConnectionCounters {
    /// The effective connection limits.
    limits: ConnectionLimits,
    /// The current number of incoming connections.
    pending_incoming: u32,
    /// The current number of outgoing connections.
    pending_outgoing: u32,
    /// The current number of established inbound connections.
    established_incoming: u32,
    /// The current number of established outbound connections.
    established_outgoing: u32,
}

impl Default for ConnectionCounters {
    fn default() -> Self {
        Self {
            limits: ConnectionLimits::default(),
            pending_incoming: 4,
            pending_outgoing: 4,
            established_incoming: 4,
            established_outgoing: 4
        }
    }

}

#[derive(Debug, Clone, Default)]
pub struct ConnectionLimits {
    max_pending_incoming: Option<u32>,
    max_pending_outgoing: Option<u32>,
    max_established_incoming: Option<u32>,
    max_established_outgoing: Option<u32>,
    max_established_per_peer: Option<u32>,
    max_established_total: Option<u32>,
}