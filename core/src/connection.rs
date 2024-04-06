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
pub struct Connection<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> {
    pub id: ConnectionId<Id>,
    pub source_type: NodeConfig,
}

impl<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> PartialEq for Connection<Id> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> Eq for Connection<Id> {}

impl<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>> Hash for Connection<Id> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ConnectionId<Id: Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id>>(pub Id);

impl<Id: Default + From<bool> + Eq + Hash + PartialEq + Copy + Debug + Add<Output = Id > > ConnectionId<Id> {
    /// Creates a unique id for a new connection
    pub fn new(id: Id) -> ConnectionId<Id> {
        Self(id)
    }

    pub fn incr(&mut self) -> Self {
        self.0 = self.0 + Id::from(true); // This is a hack to have a *one* element for the generic type Id.
        *self
    }
}

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

#[test]
fn from_true_is_one_for_scalars() {
    assert_eq!(ConnectionId::<i8>::default().incr(), ConnectionId::<i8>(1));
    assert_eq!(ConnectionId::<i16>::default().incr(), ConnectionId::<i16>(1));
    assert_eq!(ConnectionId::<i32>::default().incr(), ConnectionId::<i32>(1));
    assert_eq!(ConnectionId::<i64>::default().incr(), ConnectionId::<i64>(1));
    assert_eq!(ConnectionId::<i128>::default().incr(), ConnectionId::<i128>(1));
    assert_eq!(ConnectionId::<isize>::default().incr(), ConnectionId::<isize>(1));
    assert_eq!(ConnectionId::<u8>::default().incr(), ConnectionId::<u8>(1));
    assert_eq!(ConnectionId::<u16>::default().incr(), ConnectionId::<u16>(1));
    assert_eq!(ConnectionId::<u32>::default().incr(), ConnectionId::<u32>(1));
    assert_eq!(ConnectionId::<u64>::default().incr(), ConnectionId::<u64>(1));
    assert_eq!(ConnectionId::<u128>::default().incr(), ConnectionId::<u128>(1));
    assert_eq!(ConnectionId::<usize>::default().incr(), ConnectionId::<usize>(1));
}