mod application;
mod database;
mod environment;
mod job;
mod jwt;
pub mod logging;
mod server;
mod settings;

#[allow(unused_imports)]
pub use self::{
    application::ApplicationConfig,
    database::DatabaseConfig,
    environment::{AppEnvironment, ParseEnvironmentError},
    jwt::JwtConfig,
    logging::{LogLevel, LoggingConfig},
    server::ServerConfig,
};
pub use settings::Config;

use anyhow::Result;

pub fn load() -> Result<Config> {
    Config::load()
}

#[allow(dead_code)]
pub fn load_from(path: &str) -> Result<Config> {
    Config::load_from(path)
}
