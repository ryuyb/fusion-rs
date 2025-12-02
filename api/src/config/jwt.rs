use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,

    #[serde(default = "default_jwt_issuer")]
    pub issuer: String,

    #[serde(default)]
    pub audience: Option<String>,

    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl_secs: u64,

    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl_secs: u64,
}

impl JwtConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.secret.len() < 32 {
            anyhow::bail!("JWT secret must be at least 32 characters long");
        }

        if self.access_token_ttl_secs == 0 {
            anyhow::bail!("Access token TTL must be greater than 0");
        }

        if self.refresh_token_ttl_secs == 0 {
            anyhow::bail!("Refresh token TTL must be greater than 0");
        }

        if self.access_token_ttl_secs > i64::MAX as u64 {
            anyhow::bail!("Access token TTL is too large");
        }

        if self.refresh_token_ttl_secs > i64::MAX as u64 {
            anyhow::bail!("Refresh token TTL is too large");
        }

        Ok(())
    }
}

fn default_jwt_issuer() -> String {
    "fusion".to_string()
}

fn default_access_token_ttl() -> u64 {
    15 * 60
}

fn default_refresh_token_ttl() -> u64 {
    14 * 24 * 60 * 60
}
