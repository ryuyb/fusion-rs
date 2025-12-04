pub mod check_lived;
mod types;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::{Semaphore, TryAcquireError};
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use tracing::{error, info};

use crate::AppState;
use crate::job::check_lived::CheckLivedJob;
use crate::job::types::{AppJob, JobOverlapStrategy};

pub struct JobManager {
    pub available_jobs: HashMap<String, Arc<dyn AppJob>>,
    state: Arc<AppState>,
    sched: JobScheduler,
}

impl JobManager {
    pub async fn new(state: Arc<AppState>) -> Result<Self> {
        let mut sched = JobScheduler::new()
            .await
            .context("Failed to create job manager")?;

        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Job manager shutting down");
            })
        }));

        let mut available_jobs: HashMap<String, Arc<dyn AppJob>> = HashMap::new();
        let check_lived_job = Arc::new(CheckLivedJob::new());
        available_jobs.insert(check_lived_job.name().to_string(), check_lived_job);

        Ok(Self {
            available_jobs,
            sched,
            state,
        })
    }

    pub async fn add_job(&self, cron: &str, job: Arc<dyn AppJob>) -> Result<uuid::Uuid> {
        let job_for_closure = Arc::clone(&job);
        let state_for_closure = self.state.clone();
        let job_config = job_for_closure.config();
        let concurrency_guard = Arc::new(Semaphore::new(1));
        let job_locked =
            Job::new_cron_job_async_tz(cron, chrono_tz::Asia::Shanghai, move |_uuid, _l| {
                let job = Arc::clone(&job_for_closure);
                let state = state_for_closure.clone();
                let job_guard = concurrency_guard.clone();
                let job_config = job_config;
                Box::pin(async move {
                    let _permit = match job_config.overlap_strategy {
                        JobOverlapStrategy::Skip => match job_guard.try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(TryAcquireError::NoPermits) => {
                                info!(
                                    job = job.name(),
                                    "Previous execution still running, skipping"
                                );
                                return;
                            }
                            Err(TryAcquireError::Closed) => {
                                error!(
                                    job = job.name(),
                                    "Concurrency guard unexpectedly closed, skipping"
                                );
                                return;
                            }
                        },
                        JobOverlapStrategy::Wait => match job_guard.acquire_owned().await {
                            Ok(permit) => permit,
                            Err(err) => {
                                error!(
                                    job = job.name(),
                                    ?err,
                                    "Failed to acquire concurrency guard, skipping"
                                );
                                return;
                            }
                        },
                    };

                    info!(job = job.name(), "Starting cron job");
                    match job.execute(state).await {
                        Ok(_) => info!(job = job.name(), "Cron job completed"),
                        Err(err) => error!(job = job.name(), ?err, "Cron job failed"),
                    }
                })
            })
            .context("Failed to create cron job")?;

        let guid = job_locked.guid();
        self.sched.add(job_locked).await?;

        Ok(guid)
    }

    pub async fn remove_job(&self, guid: uuid::Uuid) -> Result<()> {
        Ok(self
            .sched
            .remove(&guid)
            .await
            .context("Job remove failed")?)
    }

    pub async fn start(&self) -> Result<()> {
        Ok(self.sched.start().await.context("Job start failed")?)
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(self.sched.shutdown().await.context("Job shutdown failed")?)
    }
}
