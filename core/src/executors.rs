use crate::{
    Future,
    Pin,
    Executor,
};
use futures::{
    executor::ThreadPool,
};

pub struct CustomExecutor {
    pub executor: ThreadPool,
}

impl<T: Send + Sync + 'static> Executor<T> for CustomExecutor {
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        #[cfg(debug_assertions)]
        println!("-------------threadpool.--------------");
        self.executor.spawn_ok( async { f.await; ()})
    }
}