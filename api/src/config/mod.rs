mod config;
pub mod logging;

pub use config::Config;

use anyhow::Result;

pub fn load() -> Result<Config> {
    Config::load()
}

#[allow(dead_code)]
pub fn load_from(path: &str) -> Result<Config> {
    Config::load_from(path)
}
