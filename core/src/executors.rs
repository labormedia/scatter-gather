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
    pub executor: SyncSender<T>,
}

impl<T: Send + Sync + 'static> Executor<T> for CustomExecutor<T> {
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        const MAX_QUEUED_TASKS: usize = 10;
        let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
        #[cfg(debug_assertions)]
        println!("-------------threadpool-start.--------------");
        task_sender.clone().send(f).expect("Queue not available.");
        #[cfg(debug_assertions)]
        println!("-------------threadpool-end.--------------");
    }
}