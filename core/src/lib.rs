use std::{
    future::Future,
    pin::Pin
};

pub mod middleware_specs;
pub mod connection;
pub mod pool;

pub trait Executor<T: Send> {
    fn exec(&self, future: Pin<Box<dyn Future<Output = T> + Send>>);
}

impl<T, F: Fn(Pin<Box<dyn Future<Output = T> + Send>>)> Executor<T> for F 
where
T: Send
{
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        self(f)
    }
}

