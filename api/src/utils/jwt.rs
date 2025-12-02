use crate::config::JwtConfig;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    errors::Error as JwtLibError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct JwtUtil {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JwtClaims {
    pub user_id: i32,
    pub sub: String,
    pub iss: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtLibError),
    #[error("{0}")]
    Invalid(&'static str),
}

pub type JwtResult<T> = Result<T, JwtError>;

impl JwtUtil {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    pub fn config(&self) -> &JwtConfig {
        &self.config
    }

    pub fn generate_access_token(&self, user_id: i32) -> JwtResult<String> {
        self.generate_token(user_id, TokenType::Access)
    }

    pub fn generate_refresh_token(&self, user_id: i32) -> JwtResult<String> {
        self.generate_token(user_id, TokenType::Refresh)
    }

    pub fn decode(&self, token: &str) -> JwtResult<TokenData<JwtClaims>> {
        let validation = self.validation();
        decode::<JwtClaims>(token, &self.decoding_key, &validation).map_err(JwtError::from)
    }

    pub fn decode_access_token(&self, token: &str) -> JwtResult<JwtClaims> {
        let data = self.decode(token)?;
        if data.claims.token_type != TokenType::Access {
            return Err(JwtError::Invalid("expected access token"));
        }
        Ok(data.claims)
    }

    pub fn decode_refresh_token(&self, token: &str) -> JwtResult<JwtClaims> {
        let data = self.decode(token)?;
        if data.claims.token_type != TokenType::Refresh {
            return Err(JwtError::Invalid("expected refresh token"));
        }
        Ok(data.claims)
    }

    fn generate_token(&self, user_id: i32, token_type: TokenType) -> JwtResult<String> {
        let claims = JwtClaims::new(user_id, token_type, &self.config);
        Ok(encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &self.encoding_key,
        )?)
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        let issuer = self.config.issuer.as_str();
        validation.set_issuer(&[issuer]);

        if let Some(audience) = &self.config.audience {
            validation.set_audience(&[audience.as_str()]);
        } else {
            validation.validate_aud = false;
        }

        validation
    }
}

impl JwtClaims {
    pub fn new(user_id: i32, token_type: TokenType, config: &JwtConfig) -> Self {
        let issued_at = Utc::now();
        let duration = match token_type {
            TokenType::Access => config.access_token_ttl_secs,
            TokenType::Refresh => config.refresh_token_ttl_secs,
        };
        let expires_at = issued_at + Duration::seconds(duration as i64);

        Self {
            user_id: user_id,
            sub: user_id.to_string(),
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            exp: expires_at.timestamp(),
            iat: issued_at.timestamp(),
            token_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> JwtConfig {
        JwtConfig {
            secret: "a_very_long_secret_for_testing_only__1234567890".into(),
            issuer: "fusion-tests".into(),
            audience: Some("fusion-clients".into()),
            access_token_ttl_secs: 60,
            refresh_token_ttl_secs: 120,
        }
    }

    #[test]
    fn creates_and_validates_access_tokens() {
        let util = JwtUtil::new(config());
        let token = util
            .generate_access_token(123)
            .expect("token generation should succeed");

        let claims = util
            .decode_access_token(&token)
            .expect("token decoding should succeed");
        assert_eq!(claims.user_id, 123);
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.token_type, TokenType::Access);
        assert_eq!(claims.iss, "fusion-tests");
        assert_eq!(claims.aud.as_deref(), Some("fusion-clients"));
    }

    #[test]
    fn rejects_invalid_token_type() {
        let util = JwtUtil::new(config());
        let refresh_token = util
            .generate_refresh_token(1)
            .expect("refresh token generation should succeed");

        let err = util
            .decode_access_token(&refresh_token)
            .expect_err("should reject refresh token when expecting access token");

        matches!(err, JwtError::Invalid(_));
    }

    #[test]
    fn decodes_refresh_tokens() {
        let util = JwtUtil::new(config());
        let refresh_token = util
            .generate_refresh_token(42)
            .expect("refresh token generation should succeed");

        let claims = util
            .decode_refresh_token(&refresh_token)
            .expect("refresh token decoding should succeed");

        assert_eq!(claims.token_type, TokenType::Refresh);
    }
}
