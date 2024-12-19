mod thread_pool;

use std::num::NonZeroUsize;
use thread_pool::WorkerPool;

pub(super) struct SchedulerThread {
  workers: WorkerPool,
}

impl SchedulerThread {
  pub fn new(pool_size: NonZeroUsize) -> Self {
    Self {
      workers: WorkerPool::new(pool_size),
    }
  }
}
