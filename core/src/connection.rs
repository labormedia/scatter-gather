use crate::middleware_specs::*;
use std::{
    fmt,
    error,
    task::{
        Poll,
        Context
    },
};

pub struct Connection<THandler: ConnectionHandler> {
    id: ConnectionId,
    source_type: ServerConfig<THandler>,
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
pub enum ConnectionHandlerInEvent {
    Connect,
    Disconnect
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
    ) -> Poll<
        ConnectionHandlerOutEvent<Self::OutEvent>
    >;

    fn inject_event(&mut self, event: Self::InEvent);
}