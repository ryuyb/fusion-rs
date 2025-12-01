use super::logging::LoggingConfig;
use anyhow::Context;
use config::{Config as ConfigBuilder, Environment, File as ConfigFile};
use serde::Deserialize;
use std::{env, str::FromStr};
use tracing::log;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    Development,
    Production,
    Test,
}

impl AppEnvironment {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEnvironment::Development => "development",
            AppEnvironment::Production => "production",
            AppEnvironment::Test => "test",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseEnvironmentError;

impl FromStr for AppEnvironment {
    type Err = ParseEnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" => Ok(AppEnvironment::Development),
            "production" => Ok(AppEnvironment::Production),
            "test" => Ok(AppEnvironment::Test),
            _ => Err(ParseEnvironmentError),
        }
    }
}

impl Default for AppEnvironment {
    fn default() -> Self {
        AppEnvironment::Development
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub environment: AppEnvironment,

    pub application: ApplicationConfig,

    pub server: ServerConfig,

    pub database: DatabaseConfig,

    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    pub workers: Option<usize>,

    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    #[serde(default = "default_max_request_size")]
    pub max_request_size_bytes: usize,

    #[serde(default = "default_graceful_shutdown_timeout")]
    pub graceful_shutdown_timeout_secs: u64,
}

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
}

// ============ Default value fn  ============
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_request_timeout() -> u64 {
    30
}
fn default_max_request_size() -> usize {
    10 * 1024 * 1024
} // 10MB
fn default_graceful_shutdown_timeout() -> u64 {
    30
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
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let run_env = env::var("FUSION_APP_ENV").unwrap_or_else(|_| "development".into());

        let config_dir = env::var("FUSION_CONFIG_DIR").unwrap_or_else(|_| "config".into());

        let configs = ConfigBuilder::builder()
            .add_source(ConfigFile::with_name(&format!("{}/default", config_dir)).required(false))
            .add_source(
                ConfigFile::with_name(&format!("{}/{}", config_dir, run_env)).required(false),
            )
            .add_source(
                ConfigFile::with_name(&format!("{}/local", config_dir).as_ref()).required(false),
            )
            .add_source(
                Environment::with_prefix("FUSION")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()
            .context("Failed to build configuration")?;

        let settings: Config = configs
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        settings.validate()?;

        Ok(settings)
    }

    pub fn load_from(path: &str) -> anyhow::Result<Self> {
        let settings = ConfigBuilder::builder()
            .add_source(ConfigFile::with_name(path))
            .add_source(
                Environment::with_prefix("FUSION")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()
            .context("Failed to build configuration")?;

        let settings: Config = settings
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        settings.validate()?;

        Ok(settings)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.server.port == 0 {
            anyhow::bail!("Invalid server port: {}", self.server.port);
        }

        self.logging.validate()?;

        Ok(())
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
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
            .sqlx_logging(false)
            .sqlx_logging_level(log::LevelFilter::Info)
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
        if let Some(idx) = self.url.find('@') {
            if let Some(start) = self.url[..idx].rfind(':') {
                let mut masked = self.url.clone();
                masked.replace_range(start + 1..idx, "****");
                return masked;
            }
        }
        self.url.clone()
    }
}
