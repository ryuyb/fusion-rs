mod application;
mod config;
mod database;
mod environment;
mod jwt;
pub mod logging;
mod server;

#[allow(unused_imports)]
pub use self::{
    application::ApplicationConfig,
    database::DatabaseConfig,
    environment::{AppEnvironment, ParseEnvironmentError},
    jwt::JwtConfig,
    logging::{LogLevel, LoggingConfig},
    server::ServerConfig,
};
pub use config::Config;

use anyhow::Result;

pub fn load() -> Result<Config> {
    Config::load()
}

#[allow(dead_code)]
pub fn load_from(path: &str) -> Result<Config> {
    Config::load_from(path)
}
