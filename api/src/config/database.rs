use super::logging::LogLevel as LoggingLevel;
use anyhow::Context;
use serde::Deserialize;
use tracing::log;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,

    #[serde(default = "default_db_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_db_min_connections")]
    pub min_connections: u32,

    #[serde(default = "default_db_connect_timeout")]
    pub connect_timeout_secs: u64,

    #[serde(default = "default_db_idle_timeout")]
    pub idle_timeout_secs: u64,

    #[serde(default = "default_db_max_lifetime")]
    pub max_lifetime_secs: u64,

    #[serde(default = "default_true")]
    pub run_migrations: bool,

    #[serde(default)]
    pub sqlx_logging: bool,

    #[serde(default = "default_sqlx_log_level")]
    pub sqlx_logging_level: LoggingLevel,
}

impl DatabaseConfig {
    pub async fn create_db(&self) -> anyhow::Result<sea_orm::DatabaseConnection> {
        use sea_orm::{ConnectOptions, Database};
        use std::time::Duration;

        tracing::info!(
            "Creating database connection pool (max_connections: {})",
            self.max_connections
        );

        let opt = ConnectOptions::new(&self.url)
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.connect_timeout_secs))
            .idle_timeout(Some(Duration::from_secs(self.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(self.max_lifetime_secs)))
            .sqlx_logging(self.sqlx_logging)
            .sqlx_logging_level(logging_level_to_log_filter(self.sqlx_logging_level))
            .to_owned();
        let db = Database::connect(opt)
            .await
            .context("Database connection failed")?;

        if self.run_migrations {
            tracing::info!("Running database migrations");
            use migration::{Migrator, MigratorTrait};

            Migrator::up(&db, None)
                .await
                .context("Failed to run migrate")?;
        }

        Ok(db)
    }

    pub fn masked_url(&self) -> String {
        if let Some(idx) = self.url.find('@')
            && let Some(start) = self.url[..idx].rfind(':')
        {
            let mut masked = self.url.clone();
            masked.replace_range(start + 1..idx, "****");
            return masked;
        }
        self.url.clone()
    }
}

fn default_db_max_connections() -> u32 {
    10
}

fn default_db_min_connections() -> u32 {
    2
}

fn default_db_connect_timeout() -> u64 {
    30
}

fn default_db_idle_timeout() -> u64 {
    600
}

fn default_db_max_lifetime() -> u64 {
    1800
}

fn default_true() -> bool {
    true
}

fn default_sqlx_log_level() -> LoggingLevel {
    LoggingLevel::Info
}

fn logging_level_to_log_filter(level: LoggingLevel) -> log::LevelFilter {
    match level {
        LoggingLevel::Error => log::LevelFilter::Error,
        LoggingLevel::Warn => log::LevelFilter::Warn,
        LoggingLevel::Info => log::LevelFilter::Info,
        LoggingLevel::Debug => log::LevelFilter::Debug,
        LoggingLevel::Trace => log::LevelFilter::Trace,
    }
}
