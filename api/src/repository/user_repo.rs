use crate::dto::CreateUserDto;
use crate::error::{AppResult, IntoAppResult};
use entity::prelude::User;
use entity::user::{ActiveModel, Model};
use sea_orm::{DbConn, EntityTrait, Set};

pub struct UserRepository {
    db: DbConn,
}

impl UserRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn create(&self, data: &CreateUserDto) -> AppResult<Model> {
        let user = ActiveModel {
            id: Default::default(),
            username: Set(data.username.clone()),
            email: Set(data.email.clone()),
            password: Set(data.password.clone()),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        User::insert(user)
            .exec_with_returning(&self.db)
            .await
            .into_app_result()
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<Model>> {
        User::find_by_id(id).one(&self.db).await.into_app_result()
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<Model>> {
        User::find_by_username(username)
            .one(&self.db)
            .await
            .into_app_result()
    }

    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<Model>> {
        User::find_by_email(email)
            .one(&self.db)
            .await
            .into_app_result()
    }
}
