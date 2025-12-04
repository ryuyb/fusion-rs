use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::job::types::{AppJob, JobOverlapStrategy};

#[derive(Default)]
pub struct JobRegistry {
    jobs: HashMap<&'static str, RegisteredJob>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_arc(&mut self, job: Arc<dyn AppJob>) -> &mut Self {
        let handle = RegisteredJob::new(job);
        self.jobs.insert(handle.name(), handle);
        self
    }

    pub fn register<J>(&mut self, job: J) -> &mut Self
    where
        J: AppJob,
    {
        self.register_arc(Arc::new(job))
    }

    pub fn get(&self, name: &str) -> Option<&RegisteredJob> {
        self.jobs.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &RegisteredJob)> {
        self.jobs.iter().map(|(name, job)| (*name, job))
    }
}

pub struct RegisteredJob {
    job: Arc<dyn AppJob>,
    guard: Arc<Semaphore>,
}

impl RegisteredJob {
    fn new(job: Arc<dyn AppJob>) -> Self {
        Self {
            job,
            guard: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn name(&self) -> &'static str {
        self.job.name()
    }

    pub fn job(&self) -> Arc<dyn AppJob> {
        Arc::clone(&self.job)
    }

    pub fn guard(&self) -> Arc<Semaphore> {
        Arc::clone(&self.guard)
    }

    pub fn overlap_strategy(&self) -> JobOverlapStrategy {
        self.job.config().overlap_strategy
    }
}
