use crate::middleware_specs::*;
use std::{
    fmt,
    task::{
        Poll,
        Context
    },
};

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub source_type: ServerConfig,
    // pub handler: THandler
    // pub handler: THandler
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
    Disconnect,
    Intercept()
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

    fn inject_event(&mut self, event: Self::InEvent);
    fn eject_event(&mut self, event: Self::OutEvent) -> Self::OutEvent;
}

impl<'b> ConnectionHandler<'b> for Connection {
    type InEvent = ConnectionHandlerInEvent;
    type OutEvent = ConnectionHandlerOutEvent<Connection>;

    fn poll(
            self,
            cx: &mut Context<'_>,
        ) -> Poll<Self::OutEvent> 
    {
        #[cfg(debug_assertions)]
        println!("self {:?}", self);
        Poll::Pending
    }

    fn inject_event(&mut self, event: Self::InEvent) {
        #[cfg(debug_assertions)]
        println!("Injecting event on Connection. {:?}", event);
    }

    fn eject_event(&mut self, event: Self::OutEvent) -> Self::OutEvent {
        event
    }
}