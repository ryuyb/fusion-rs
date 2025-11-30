use crate::dto::{CreateUserDto, UserDto};
use crate::error::{AppResult, Entity, IntoAppResult};
use crate::repository::UserRepository;
use entity::user::Model;
use std::sync::Arc;

pub struct UserService {
    repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>) -> UserService {
        UserService { repo: user_repo }
    }

    pub async fn create(&self, user: CreateUserDto) -> AppResult<UserDto> {
        if self
            .repo
            .find_by_username(user.username.as_str())
            .await?
            .is_some()
        {
            return Err(Model::duplicated_by("username", user.username));
        }
        if self
            .repo
            .find_by_email(user.email.as_str())
            .await?
            .is_some()
        {
            return Err(Model::duplicated_by("email", user.email));
        }
        Ok(self.repo.create(&user).await?.into())
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<UserDto> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Model::not_found_by("id", id))
            .map(|model| model.into())
            .into_app_result()
    }
}
