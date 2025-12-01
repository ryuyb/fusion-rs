use crate::dto::CreateUserDto;
use crate::error::{AppResult, IntoAppResult};
use entity::prelude::User;
use entity::user::{ActiveModel, Model};
use sea_orm::{DbConn, EntityTrait, PaginatorTrait, Set};

pub struct UserRepository {
    db: DbConn,
}

impl UserRepository {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    pub async fn create(&self, data: &CreateUserDto, password_hash: &str) -> AppResult<Model> {
        let user = ActiveModel {
            id: Default::default(),
            username: Set(data.username.clone()),
            email: Set(data.email.clone()),
            password: Set(password_hash.to_string()),
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

    pub async fn list(&self, page: u64, page_size: u64) -> AppResult<(u64, Vec<Model>)> {
        let paginator = User::find().paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let user_list = paginator.fetch_page(page).await?;
        Ok((total, user_list))
    }
}
