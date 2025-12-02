use crate::dto::{CreateUserDto, RegisterRequest};
use crate::error::{AppError, AppResult};
use crate::repository::UserRepository;
use crate::utils::{jwt::JwtUtil, password};
use anyhow::Context;
use chrono::{Duration, NaiveDateTime, Utc};
use entity::user::Model;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
    pub refresh_expires_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy)]
pub enum LoginIdentifier<'a> {
    Username(&'a str),
    Email(&'a str),
}

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    jwt: Arc<JwtUtil>,
}

impl AuthService {
    pub fn new(user_repo: Arc<UserRepository>, jwt: Arc<JwtUtil>) -> Self {
        Self { user_repo, jwt }
    }

    pub async fn register(&self, data: RegisterRequest) -> AppResult<AuthTokens> {
        let create_user: CreateUserDto = data.into();
        let hashed_password =
            password::hash_password(&create_user.password).context("Failed to hash password")?;
        let user = self
            .user_repo
            .create(&create_user, &hashed_password)
            .await?;

        self.issue_tokens(user.id)
    }

    pub async fn authenticate(
        &self,
        identifier: LoginIdentifier<'_>,
        password: &str,
    ) -> AppResult<AuthTokens> {
        let user = match identifier {
            LoginIdentifier::Username(username) => {
                self.user_repo.find_by_username(username).await?
            }
            LoginIdentifier::Email(email) => self.user_repo.find_by_email(email).await?,
        }
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

        self.authenticate_user(user, password).await
    }

    pub async fn refresh_tokens(&self, refresh_token: &str) -> AppResult<AuthTokens> {
        let claims = self
            .jwt
            .decode_refresh_token(refresh_token)
            .context("Failed to decode refresh token")?;

        let user = self
            .user_repo
            .find_by_id(claims.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".into()))?;

        self.issue_tokens(user.id)
    }

    async fn authenticate_user(&self, user: Model, password: &str) -> AppResult<AuthTokens> {
        let password_matches = password::verify_password(password, &user.password)
            .context("Failed to verify password hash")?;

        if !password_matches {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        self.issue_tokens(user.id)
    }

    fn issue_tokens(&self, user_id: i32) -> AppResult<AuthTokens> {
        let access_token = self
            .jwt
            .generate_access_token(user_id)
            .context("Failed to create access token")?;
        let refresh_token = self
            .jwt
            .generate_refresh_token(user_id)
            .context("Failed to create refresh token")?;
        let config = self.jwt.config();
        let now = Utc::now();
        let access_exp = now + Duration::seconds(config.access_token_ttl_secs as i64);
        let refresh_exp = now + Duration::seconds(config.refresh_token_ttl_secs as i64);

        Ok(AuthTokens {
            user_id,
            access_token,
            refresh_token,
            expires_at: access_exp.naive_utc(),
            refresh_expires_at: refresh_exp.naive_utc(),
        })
    }
}
