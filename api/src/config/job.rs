use anyhow::Context;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct JobConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cron_expr: String,
}

impl JobConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        cron::Schedule::from_str(&self.cron_expr)
            .with_context(|| format!("Invalid cron expression: {}", self.cron_expr))?;
        Ok(())
    }
}
