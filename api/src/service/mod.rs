mod user_service;

use crate::repository::Repositories;
use anyhow::Result;
use std::sync::Arc;
pub use user_service::UserService;

#[derive(Clone)]
pub struct Services {
    pub user: Arc<UserService>,
}

impl Services {
    pub async fn build(repos: Arc<Repositories>) -> Result<Self> {
        let user_service = Arc::new(UserService::new(repos.user.clone()));

        Ok(Self { user: user_service })
    }
}
