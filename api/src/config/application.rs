use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
}
