use futures_util::{StreamExt};
use tokio::io::{AsyncReadExt};
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use std::{
    fmt,
    task::{
        Poll,
        Context
    },
    error
};
use futures::{
    channel::mpsc,
    SinkExt
};

pub mod middleware_specs;
pub mod connection;

use self::connection::*;
use self::middleware_specs::*;

pub trait Executor {
    fn exec(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);
}

impl<F: Fn(Pin<Box<dyn Future<Output = ()> + Send>>)> Executor for F {
    fn exec(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self(f)
    }
}

pub enum ConnectionHandlerEvent<TCustom, TError> {
    Close(TError),
    Custom(TCustom)
}

pub trait ConnectionHandler: Interceptor + Send + 'static {

    type InEvent: fmt::Debug + Send + 'static;
    type OutEvent: fmt::Debug + Send + 'static;
    type Error: error::Error + fmt::Debug + Send + 'static;

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<
        ConnectionHandlerEvent<Self::OutEvent, Self::Error>
    >;

    fn inject_event(&mut self, event: Self::InEvent);
}

/// Pool Connection is initialized with the handler defined 
/// for the specific Connection::source_type::handler.
pub struct PoolConnection<THandler: ConnectionHandler> {
    conn: Pin<Box<Connection<THandler>>>,
    handler: THandler
}

pub enum PoolEvent<THandler: ConnectionHandler> {
    ConnectionEstablished(PoolConnection<THandler>),
    ConnectionClosed(PoolConnection<THandler>),
    ConnectionEvent(PoolConnection<THandler>),
}


impl<THandler> PoolEvent<THandler> 
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
            _ => events.try_send(self)
        }
    }
}

pub struct Pool<THandler: ConnectionHandler> {
    local_id: usize,
    counters: ConnectionCounters,
    pending: HashMap<ConnectionId, PendingConnection<THandler>>,
    established: HashMap<ConnectionId, EstablishedConnection<THandler>>,
    executor: Option<Box<dyn Executor + Send>>
}

pub struct PendingConnection<THandler: ConnectionHandler>(PoolConnection<THandler>);

pub struct EstablishedConnection<THandler: ConnectionHandler>(PoolConnection<THandler>);

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

#[derive(Debug, Clone, Default)]
pub struct ConnectionLimits {
    max_pending_incoming: Option<u32>,
    max_pending_outgoing: Option<u32>,
    max_established_incoming: Option<u32>,
    max_established_outgoing: Option<u32>,
    max_established_per_peer: Option<u32>,
    max_established_total: Option<u32>,
}