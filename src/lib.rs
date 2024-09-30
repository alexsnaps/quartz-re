/*
 * Copyright 2024 Alex Snaps
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::collections::BTreeSet;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

pub struct Scheduler {
  running: Arc<AtomicBool>,
  job_store: Arc<JobStore>,
  scheduler_thread: ManuallyDrop<JoinHandle<()>>,
}

impl Scheduler {
  pub fn new() -> Self {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    let job_store = Arc::new(JobStore::new());
    let store = Arc::clone(&job_store);

    let handle = thread::Builder::new()
      .name("Quartz Scheduler Thread".to_string())
      .spawn(move || {
        while r.load(Ordering::SeqCst) {
          if let Some(job) = store.next_job() {
            job.execute();
          }
        }
      })
      .expect("");

    Self {
      running,
      job_store,
      scheduler_thread: ManuallyDrop::new(handle),
    }
  }

  pub fn schedule_job(&mut self, _job: JobDetail, _trigger: Trigger) {
    self.job_store.signal();
  }
}

impl Drop for Scheduler {
  fn drop(&mut self) {
    if self
      .running
      .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
      .is_ok()
    {
      unsafe {
        let handle = ManuallyDrop::take(&mut self.scheduler_thread);
        handle.join().expect("Couldn't join the scheduler thread");
      }
    }
  }
}

impl Default for Scheduler {
  fn default() -> Self {
    Self::new()
  }
}

pub struct JobDetail {
  id: String,
  group: String,
  target_fn: fn(),
}

impl JobDetail {
  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn group(&self) -> &str {
    &self.group
  }

  pub fn execute(&self) {
    (self.target_fn)();
  }
}

impl JobDetail {
  pub fn with_identity<S: Into<String>>(id: S, group: S, target: fn()) -> Self {
    Self {
      id: id.into(),
      group: group.into(),
      target_fn: target,
    }
  }
}

pub struct Trigger {
  id: String,
  group: String,
  #[allow(dead_code)]
  start_time: SystemTime,
}

impl Trigger {
  pub fn with_identity<S: Into<String>>(id: S, group: S) -> Self {
    Self {
      id: id.into(),
      group: group.into(),
      start_time: SystemTime::now(),
    }
  }

  pub fn start_at(self, start_time: SystemTime) -> Self {
    Self {
      id: self.id,
      group: self.group,
      start_time,
    }
  }
}

struct JobStore {
  signal: Arc<Condvar>,
  #[allow(dead_code)]
  data: Arc<Mutex<BTreeSet<String>>>,
}

impl JobStore {
  fn new() -> Self {
    Self {
      signal: Arc::new(Default::default()),
      data: Arc::new(Mutex::new(Default::default())),
    }
  }

  fn next_job(&self) -> Option<JobDetail> {
    None
  }

  fn signal(&self) {
    self.signal.notify_one()
  }
}

impl Default for JobStore {
  fn default() -> Self {
    JobStore::new()
  }
}

#[cfg(test)]
mod tests {
  use crate::{JobDetail, Scheduler, Trigger};
  use std::thread;
  use std::time::{Duration, SystemTime};

  #[test]
  #[ignore]
  fn test_basic_api() {
    // First we must get a reference to a scheduler
    let mut sched = Scheduler::new();

    // computer a time that is a second from now
    let run_time = SystemTime::now() + Duration::from_secs(1);

    println!("------- Scheduling Job  -------------------");

    // define the job and tie it to our HelloJob class
    let job_id = "job1";
    let job = JobDetail::with_identity(job_id, "group1", || println!("Hello world!"));

    // Trigger the job to run on the next round minute
    let trigger = Trigger::with_identity("trigger1", "group1").start_at(run_time);

    // Tell quartz to schedule the job using our trigger
    sched.schedule_job(job, trigger);
    println!("{job_id} will run at: {run_time:?}");

    // wait long enough so that the scheduler as an opportunity to
    // run the job!
    println!("------- Waiting 65 seconds... -------------");
    // wait 2 seconds to show job
    thread::sleep(Duration::from_secs(2));
    // executing...

    // shut down the scheduler
    println!("------- Shutting Down ---------------------");
  }
}
