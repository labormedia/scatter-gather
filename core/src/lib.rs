
pub use core::{
    future::Future,
    pin::Pin
};

pub mod middleware_interface;
pub mod connection;
pub mod pool;
pub mod executors;

pub trait Executor<T: Send + Sync> {
    fn exec(&self, future: Pin<Box<dyn Future<Output = T> + Send>>);
}

impl<T: Sync, F: Fn(Pin<Box<dyn Future<Output = T> + Send>>)> Executor<T> for F 
where
T: Send
{
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        self(f)
    }
}

