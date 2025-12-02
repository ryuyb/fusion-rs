use serde::Deserialize;

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
