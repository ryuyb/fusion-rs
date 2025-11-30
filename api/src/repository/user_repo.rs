use crate::error::{AppResult, IntoAppResult};
use entity::prelude::User;
use entity::user::Model;
use sea_orm::{DbConn, EntityTrait};

pub struct UserRepository {
    db: DbConn,
}

impl UserRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<Model>> {
        User::find_by_id(id).one(&self.db).await.into_app_result()
    }
}
