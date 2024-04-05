use crate::middleware_interface::*;
use core::{
    ops::Add,
    task::{
        Poll,
        Context,
    },
};
use std::{
    fmt::Debug,
    hash::{
        Hash,
        Hasher
    },
    error::Error,
};

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId<usize>,
    pub source_type: NodeConfig,
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
pub struct ConnectionId<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>>(pub Id);

impl<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> ConnectionId<Id> {
    /// Creates a unique id for a new connection
    pub fn new(id:Id) -> ConnectionId<Id> {
        Self(id)
    }
}

impl ConnectionId<i32> {}
impl ConnectionId<usize> {}

impl<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> Add<Id> for ConnectionId<Id>
where 
Id: Eq + Hash + PartialEq + Copy + Debug,
{
    type Output = Self;

    fn add(self, other: Id) -> Self {
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

    type InEvent: Debug + Send + 'a;
    type OutEvent: Debug + Send + 'a;
    // type Error: error::Error + Debug + Send + 'static;

    fn poll(
        self,
        cx: &mut Context<'_>,
    ) -> Poll<Self::OutEvent>;

    fn inject_event(&mut self, event: Self::InEvent) -> Result<(), Box<dyn std::error::Error>>;
    fn eject_event(&mut self, event: Self::OutEvent) -> Result<(), Box<dyn std::error::Error>>;
    fn as_any(&self) -> &dyn std::any::Any;
}