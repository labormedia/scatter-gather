use futures::future::Pending;

use crate::middleware_specs::*;
use std::{
    fmt,
    error,
    task::{
        Poll,
        Context
    },
};

#[derive(Debug)]
pub struct Connection<THandler: ConnectionHandler> {
    pub id: ConnectionId,
    pub source_type: ServerConfig<THandler>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionId(usize);

impl ConnectionId {
    /// Creates a unique id for a new connection
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub enum ConnectionHandlerInEvent<T> {
    Connect,
    Disconnect,
    Intercept(T)
}

#[derive(Debug)]
pub enum ConnectionHandlerOutEvent<TCustom> {
    ConnectionEstablished(TCustom),
    ConnectionClosed(TCustom),
    ConnectionEvent(TCustom),
    // ConnectionError(TError)
}

pub trait ConnectionHandler: Send + 'static {

    type InEvent: fmt::Debug + Send + 'static;
    type OutEvent: fmt::Debug + Send + 'static;
    // type Error: error::Error + fmt::Debug + Send + 'static;

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Self::OutEvent>;

    fn inject_event(&mut self, event: Self::InEvent);
    fn eject_event(&mut self, event: Self::OutEvent);
}

impl<THandler: fmt::Debug + Send + ConnectionHandler> ConnectionHandler for Connection<THandler> {
    type InEvent = ConnectionHandlerInEvent<THandler>;
    type OutEvent = ConnectionHandlerOutEvent<THandler>;

    fn poll(
            &mut self,
            cx: &mut Context<'_>,
        ) -> Poll<Self::OutEvent> {
        Poll::Pending
    }

    fn inject_event(&mut self, event: Self::InEvent) {
        
    }

    fn eject_event(&mut self, event: Self::OutEvent) {
        
    }
}