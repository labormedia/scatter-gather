use crate::middleware_interface::*;
use core::task::{
    Poll,
    Context,
};
use std::{
    fmt,
    hash::{
        Hash,
        Hasher
    },
    error::Error,
};

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub source_type: NodeConfig,
    // pub handler: THandler
    // pub handler: THandler
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionId(usize);

impl ConnectionId {
    /// Creates a unique id for a new connection
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl std::ops::Add<usize> for ConnectionId {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self(self.0 + other)
    }
}

#[derive(Debug)]
pub enum ConnectionHandlerInEvent {
    Connect,
    Disconnect,
    Intercept
}

#[derive(Debug)]
pub enum ConnectionHandlerOutEvent<TCustom> {
    ConnectionEstablished(TCustom),
    ConnectionClosed(TCustom),
    ConnectionEvent(TCustom),
    Custom
    // ConnectionError(TError)
}


pub trait ConnectionHandler<'a>: 'a + Send {

    type InEvent: fmt::Debug + Send + 'a;
    type OutEvent: fmt::Debug + Send + 'a;
    // type Error: error::Error + fmt::Debug + Send + 'static;

    fn poll(
        self,
        cx: &mut Context<'_>,
    ) -> Poll<Self::OutEvent>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>>;
    fn eject_event(&mut self, event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>>;
    fn as_any(&self) -> &dyn std::any::Any;
}