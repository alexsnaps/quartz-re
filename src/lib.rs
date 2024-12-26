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
mod threading;

use crate::threading::SchedulerThread;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime};

pub struct Scheduler {
  job_store: Arc<JobStore>,
  scheduler_thread: SchedulerThread,
}

impl Scheduler {
  pub fn new() -> Self {
    let job_store = Arc::new(JobStore::new());
    let scheduler_thread = SchedulerThread::new(NonZeroUsize::new(2).unwrap(), Arc::clone(&job_store));

    Self {
      job_store,
      scheduler_thread,
    }
  }

  pub fn schedule_job(&mut self, _job: JobDetail, _trigger: Trigger) {
    self.job_store.signal();
  }

  pub fn shutdown(self) {
    self.scheduler_thread.shutdown();
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

impl From<JobDetail> for () {
  fn from(_value: JobDetail) -> Self {
    // todo for an actual useful type
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

  fn next_fire(&self) -> Option<Duration> {
    Some(Duration::from_millis(1))
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
    println!("------- Waiting 2 seconds... -------------");
    // wait 2 seconds to show job
    thread::sleep(Duration::from_secs(2));
    // executing...

    // shut down the scheduler
    println!("------- Shutting Down ---------------------");
    sched.shutdown();
  }
}
