use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub(super) struct WorkerPool {
  running: Arc<AtomicBool>,
  workers: Vec<(Arc<Worker>, JoinHandle<()>)>,
}

struct Worker {
  busy: AtomicBool,
  task: Mutex<Option<()>>,
  cvar: Condvar,
}

impl WorkerPool {
  pub fn new(size: NonZeroUsize) -> Self {
    let size = size.get();

    let running = Arc::new(AtomicBool::new(true));
    let mut workers = Vec::with_capacity(size);

    for _ in 0..size {
      let running = running.clone();
      let worker: Arc<Worker> = Arc::default();
      let w = worker.clone();
      let jh = thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
          worker.do_work();
        }
      });
      workers.push((w, jh));
    }

    Self { running, workers }
  }

  pub fn shutdown(mut self) {
    self.running.store(false, Ordering::SeqCst);
    for (worker, handle) in self.workers.drain(..) {
      worker.wake_up();
      handle.join().expect("Worker thread panicked");
    }
  }
}

impl Drop for WorkerPool {
  fn drop(&mut self) {
    if !self.workers.is_empty() {
      eprintln!("WorkerPool hasn't been shutdown prior to being Dropped!");
    }
  }
}

impl Worker {
  fn new() -> Self {
    Self {
      busy: Default::default(), // todo non atomic? if only accessed from within the scheduler's lock
      task: Default::default(),
      cvar: Default::default(),
    }
  }

  fn assign(&self, work: ()) -> Result<(), ()> {
    if self.busy.load(Ordering::Acquire) {
      return Err(());
    }
    let mut task = self.task.lock().unwrap();
    match *task {
      None => {
        *task = Some(work);
        self.busy.store(true, Ordering::Release);
        self.cvar.notify_one();
        Ok(())
      },
      Some(_) => Err(()),
    }
  }

  fn busy(&self) -> bool {
    self.busy.load(Ordering::Acquire)
  }

  fn do_work(&self) {
    let mut task = self.task.lock().unwrap();
    while task.is_none() {
      // if task == None && busy == true => We're shutting down!
      if self
        .busy
        .compare_exchange(true, false, Ordering::Release, Ordering::Acquire)
        .is_ok()
      {
        return;
      }
      // otherwise we just started, or it's a spurious wakeup, wait...
      task = self.cvar.wait(task).unwrap();
    }
    task.take().expect("task must be available");
    // todo do the work
    self.busy.store(false, Ordering::Release);
  }

  fn wake_up(&self) {
    let l = self.task.lock().unwrap();
    self.busy.store(true, Ordering::Release);
    self.cvar.notify_one();
    std::mem::drop(l);
  }
}

impl Default for Worker {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread;

  #[test]
  fn test_thread_pool() {
    let pool = WorkerPool::new(NonZeroUsize::new(1).unwrap());
    pool.shutdown();
  }

  #[test]
  fn worker_api() {
    let worker = Worker::new();
    assert!(!worker.busy());
    assert!(worker.assign(()).is_ok());
    assert!(worker.busy());
    assert!(worker.assign(()).is_err());

    let worker = thread::spawn(move || {
      worker.do_work();
      worker
    })
    .join()
    .unwrap();

    assert!(!worker.busy());
    assert!(worker.assign(()).is_ok());
  }
}
