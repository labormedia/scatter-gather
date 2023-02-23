use crate::middleware_specs::*;

pub struct Connection<TInterceptor: Interceptor> {
    id: ConnectionId,
    source_type: ServerConfig<TInterceptor>
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionId(usize);

impl ConnectionId {
    /// Creates a unique id for a new connection
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}