use sea_orm::DbConn;
use std::sync::Arc;

mod user_repo;

pub use user_repo::UserRepository;

pub struct Repositories {
    pub user: Arc<UserRepository>,
}

impl Repositories {
    pub fn new(db: DbConn) -> Self {
        Self {
            user: Arc::new(UserRepository::new(db)),
        }
    }
}
