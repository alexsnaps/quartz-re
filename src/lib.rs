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
use std::time::SystemTime;

pub struct Scheduler {}

impl Scheduler {
  pub fn start(&mut self) {
    todo!()
  }
}

impl Scheduler {
  pub fn schedule_job(&mut self, _job: JobDetail, _trigger: Trigger) {
    todo!()
  }
}

impl Default for Scheduler {
  fn default() -> Self {
    Self::new()
  }
}

impl Scheduler {
  pub fn new() -> Self {
    todo!()
  }
}

pub struct JobDetail;

impl JobDetail {
  pub fn id(&self) -> &str {
    todo!()
  }
}

impl JobDetail {
  pub fn with_identity(_id: &str, _group: &str, _function: fn()) -> Self {
    todo!()
  }
}

pub struct Trigger;

impl Trigger {
  pub fn with_identity(_id: &str, _group: &str, _start_time: SystemTime) -> Self {
    todo!()
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
    let trigger = Trigger::with_identity("trigger1", "group1", run_time);

    // Tell quartz to schedule the job using our trigger
    sched.schedule_job(job, trigger);
    println!("{job_id} will run at: {run_time:?}");

    // Start up the scheduler (nothing can actually run until the
    // scheduler has been started)
    sched.start();

    println!("------- Started Scheduler -----------------");

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
