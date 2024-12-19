mod thread_pool;

use crate::JobStore;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Acquire;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use thread_pool::WorkerPool;

const DEFAULT_WAIT_NO_WORK: Duration = Duration::from_secs(1);

pub(super) struct SchedulerThread {
  halted: Arc<AtomicBool>,
  workers: Arc<WorkerPool<()>>,
  store: Arc<JobStore>,
  handle: JoinHandle<()>,
}

impl SchedulerThread {
  pub fn new(pool_size: NonZeroUsize, store: Arc<JobStore>) -> Self {
    let halted = Arc::new(AtomicBool::default());
    let workers = Arc::new(WorkerPool::new(pool_size));
    let handle = {
      let workers = workers.clone();
      let halted = halted.clone();
      let store = store.clone();

      thread::spawn(move || {
        while !halted.load(Acquire) {
          let available_workers = workers.available_workers();
          // todo delete this!
          if cfg!(test) {
            println!("We have {} workers available", available_workers);
          }
          let next_fire = store.next_fire().unwrap_or(DEFAULT_WAIT_NO_WORK);
          thread::sleep(next_fire);
        }
      })
    };

    // todo delete this!
    if cfg!(test) {
      workers.submit(()).unwrap();
      thread::sleep(Duration::from_millis(1));
      workers.submit(()).unwrap();
      thread::sleep(Duration::from_millis(1));
      workers.submit(()).unwrap();
    }

    Self {
      halted,
      workers,
      store,
      handle,
    }
  }

  pub fn shutdown(self) {
    self.halted.store(true, std::sync::atomic::Ordering::Release);
    self.handle.join().expect("Scheduler thread panicked");
    Arc::try_unwrap(self.workers)
      .expect("worker pool is still being used!")
      .shutdown();
  }
}

#[cfg(test)]
mod tests {
  use crate::threading::SchedulerThread;
  use crate::JobStore;
  use std::num::NonZeroUsize;
  use std::sync::Arc;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn api() {
    let t = SchedulerThread::new(NonZeroUsize::new(3).unwrap(), Arc::new(JobStore::new()));
    thread::sleep(Duration::from_millis(3));
    t.shutdown();
  }
}
