use std::sync::Arc;

use anyhow::Result;
use migration::async_trait::async_trait;

use crate::AppState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobOverlapStrategy {
    Skip,
    Wait,
}

#[derive(Clone, Copy, Debug)]
pub struct JobConfig {
    pub overlap_strategy: JobOverlapStrategy,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            overlap_strategy: JobOverlapStrategy::Skip,
        }
    }
}

#[async_trait]
pub trait AppJob: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn config(&self) -> JobConfig {
        JobConfig::default()
    }

    async fn execute(&self, state: Arc<AppState>) -> Result<()>;
}
