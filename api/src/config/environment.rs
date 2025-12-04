use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    #[default]
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
