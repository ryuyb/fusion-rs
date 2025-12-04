use super::{
    application::ApplicationConfig, database::DatabaseConfig, environment::AppEnvironment,
    jwt::JwtConfig, logging::LoggingConfig, server::ServerConfig,
};
use crate::config::job::JobConfig;
use anyhow::Context;
use config::{Config as ConfigBuilder, Environment, File as ConfigFile};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub environment: AppEnvironment,
    pub application: ApplicationConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    pub jwt: JwtConfig,
    #[serde(default)]
    pub jobs: HashMap<String, JobConfig>,
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
        self.jwt.validate()?;

        for (name, job_cfg) in self.jobs.iter() {
            job_cfg
                .validate()
                .with_context(|| format!("Failed to validate job {}", name))?;
        }

        Ok(())
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
