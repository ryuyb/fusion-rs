use crate::AppState;
use crate::job::types::{AppJob, JobConfig, JobOverlapStrategy};
use migration::async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::info;

pub struct CheckLivedJob {}

impl CheckLivedJob {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AppJob for CheckLivedJob {
    fn name(&self) -> &'static str {
        "check-lived"
    }

    fn config(&self) -> JobConfig {
        JobConfig {
            overlap_strategy: JobOverlapStrategy::Skip,
        }
    }

    async fn execute(&self, _state: Arc<AppState>) -> anyhow::Result<()> {
        time::sleep(Duration::from_secs(70)).await;
        info!("check-lived {}", self.name());
        Ok(())
    }
}
