mod auth_service;
mod user_service;

use crate::repository::Repositories;
use crate::utils::jwt::JwtUtil;
use anyhow::Result;
#[allow(unused_imports)]
pub use auth_service::{AuthService, AuthTokens, LoginIdentifier};
use std::sync::Arc;
pub use user_service::UserService;

#[derive(Clone)]
pub struct Services {
    pub auth: Arc<AuthService>,
    pub user: Arc<UserService>,
}

impl Services {
    pub async fn build(repos: Arc<Repositories>, jwt: Arc<JwtUtil>) -> Result<Self> {
        let user_service = Arc::new(UserService::new(repos.user.clone()));
        let auth_service = Arc::new(AuthService::new(repos.user.clone(), jwt));

        Ok(Self {
            auth: auth_service,
            user: user_service,
        })
    }
}
