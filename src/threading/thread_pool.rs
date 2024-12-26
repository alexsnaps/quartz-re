use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug)]
pub(super) struct WorkerPool<T> {
  running: Arc<AtomicBool>,
  workers: Vec<(Arc<Worker<T>>, JoinHandle<()>)>,
}

#[derive(Debug)]
struct Worker<T> {
  busy: AtomicBool,
  task: Mutex<Option<T>>,
  cvar: Condvar,
}

impl<T: Send + 'static> WorkerPool<T> {
  pub fn new(size: NonZeroUsize) -> Self {
    let size = size.get();

    let running = Arc::new(AtomicBool::new(true));
    let mut workers = Vec::with_capacity(size);

    for i in 0..size {
      let running = running.clone();
      let worker: Arc<Worker<T>> = Arc::default();
      let w = worker.clone();
      let jh = thread::Builder::new()
        .name(format!("Quartz Worker #{i}"))
        .spawn(move || {
          while running.load(Ordering::Acquire) {
            worker.do_work();
          }
        })
        .unwrap();
      workers.push((w, jh));
    }

    Self { running, workers }
  }

  pub fn submit(&self, task: T) -> Result<(), T> {
    match self
      .workers
      .iter()
      .find(|(w, _)| !w.busy.load(Ordering::Acquire))
      .map(|(w, _)| w)
    {
      Some(w) => w.assign(task),
      None => Err(task),
    }
  }

  #[allow(dead_code)]
  pub fn available_workers(&self) -> usize {
    self.workers.iter().filter(|w| !w.0.busy()).count()
  }

  pub fn shutdown(mut self) {
    self.running.store(false, Ordering::Release);
    for (worker, handle) in self.workers.drain(..) {
      worker.wake_up();
      handle.join().expect("Worker thread panicked");
    }
  }
}

impl<T> Drop for WorkerPool<T> {
  fn drop(&mut self) {
    if !self.workers.is_empty() {
      if cfg!(test) {
        assert!(
          self.workers.is_empty(),
          "WorkerPool hasn't been shutdown prior to being Dropped!"
        );
      } else {
        eprintln!("WorkerPool hasn't been shutdown properly!");
      }
    }
  }
}

impl<T> Worker<T> {
  fn new() -> Self {
    Self {
      busy: Default::default(), // todo non atomic? if only accessed from within the scheduler's lock
      task: Default::default(),
      cvar: Default::default(),
    }
  }

  fn assign(&self, work: T) -> Result<(), T> {
    if self.busy.load(Ordering::Acquire) {
      return Err(work);
    }
    let mut task = self.task.lock().unwrap();
    match *task {
      None => {
        *task = Some(work);
        self.busy.store(true, Ordering::Release);
        self.cvar.notify_one();
        Ok(())
      },
      Some(_) => Err(work),
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
    // todo delete this!
    if cfg!(test) {
      println!("Doing work!");
      thread::sleep(Duration::from_millis(2));
      // todo do the work
      println!("Done!");
    }

    self.busy.store(false, Ordering::Release);
  }

  fn wake_up(&self) {
    let l = self.task.lock().unwrap();
    self.busy.store(true, Ordering::Release);
    self.cvar.notify_one();
    std::mem::drop(l);
  }
}

impl<T> Default for Worker<T> {
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
    let pool = WorkerPool::<()>::new(NonZeroUsize::new(1).unwrap());
    pool.shutdown();
  }

  #[test]
  fn available_workers() {
    let pool = WorkerPool::<()>::new(NonZeroUsize::new(2).unwrap());
    assert_eq!(pool.available_workers(), 2);
    pool.shutdown();
  }

  #[test]
  #[should_panic(expected = "WorkerPool hasn't been shutdown prior to being Dropped!")]
  fn test_thread_pool_dropped_panics() {
    let _pool = WorkerPool::<()>::new(NonZeroUsize::new(1).unwrap());
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
