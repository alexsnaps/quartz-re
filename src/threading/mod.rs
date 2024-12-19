mod thread_pool;

use thread_pool::WorkerPool;

pub(super) struct SchedulerThread {
  workers: WorkerPool,
}

impl SchedulerThread {
  pub fn new(pool_size: usize) -> Self {
    Self {
      workers: WorkerPool::new(pool_size),
    }
  }
}
