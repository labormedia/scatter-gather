use crate::{
    Future,
    Pin,
    Executor,
};
use futures::{
    executor::ThreadPool,
};

struct CustomExecutor {
    executor: ThreadPool,
}

impl<T: Send + Sync + 'static> Executor<T> for CustomExecutor {
    fn exec(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) {
        self.executor.spawn_ok( async { f.await; ()})
    }
}