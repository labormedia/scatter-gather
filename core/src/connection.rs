use crate::middleware_interface::*;
use std::{
    fmt,
    task::{
        Poll,
        Context
    },
    hash::{
        Hash,
        Hasher
    },
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

// A dummy implementation for complying to the architecture's boundings.
impl<'b> ConnectionHandler<'b> for Connection {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<Connection>;

    fn poll(
            self,
            _cx: &mut Context<'_>,
        ) -> Poll<Self::OutEvent> 
    {
        #[cfg(debug_assertions)]
        println!("self {:?}", self);
        Poll::Pending
    }

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        println!("Injecting event on Connection. {:?}", event);
        Ok(())
    }

    fn eject_event(&mut self, _event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as _
    }
}