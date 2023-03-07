use crate::middleware_specs::*;
use std::{
    fmt,
    task::{
        Poll,
        Context
    },
};

#[derive(Debug)]
pub struct Connection<THandler: for<'a> ConnectionHandler<'a>> {
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

impl<'b, THandler:fmt::Debug + Send + Sync + for <'a> ConnectionHandler<'a> > ConnectionHandler<'b> for Connection<THandler> {
    type InEvent = ConnectionHandlerInEvent<THandler>;
    type OutEvent = ConnectionHandlerOutEvent<THandler>;

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
        
    }

    fn eject_event(&mut self, event: Self::OutEvent) -> Self::OutEvent {
        event
    }
}