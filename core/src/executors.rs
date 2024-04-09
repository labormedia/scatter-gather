use crate::{
    Future,
    Pin,
    Executor,
};
use futures::{
    executor::ThreadPool,
};
use std::sync::{
    mpsc::{
        sync_channel,
        Receiver, 
        SyncSender,
    }
};

pub struct CustomExecutor<T> {
    executor: SyncSender<Pin<Box<(dyn futures::Future<Output = T> + std::marker::Send + 'static)>>>,
    pub receiver: Receiver<Pin<Box<(dyn futures::Future<Output = T> + std::marker::Send + 'static)>>>,
}

impl<T> CustomExecutor<T> {
    pub fn new() -> Self {
        const MAX_QUEUED_TASKS: usize = 10;
        let (executor, receiver) = sync_channel(MAX_QUEUED_TASKS);
        CustomExecutor {
            executor,
            receiver,
        }
    }
}

impl<T: Send + Sync + 'static> Executor<T> for CustomExecutor<T> {
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        
        #[cfg(debug_assertions)]
        println!("-------------threadpool-start.--------------");
        self.executor.clone().send(f).expect("Queue not available.");
        #[cfg(debug_assertions)]
        println!("-------------threadpool-end.--------------");
    }
}