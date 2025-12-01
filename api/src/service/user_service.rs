use crate::dto::{CreateUserDto, PagedResponse, UserDto};
use crate::error::{AppError, AppResult, Entity, IntoAppResult};
use crate::repository::UserRepository;
use crate::utils::password;
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
        let hashed_password = password::hash_password(&user.password).map_err(|err| {
            AppError::InternalServerError(format!("Failed to hash password: {err}"))
        })?;

        Ok(self.repo.create(&user, &hashed_password).await?.into())
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<UserDto> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Model::not_found_by("id", id))
            .map(|model| model.into())
            .into_app_result()
    }

    pub async fn list(&self, page: u64, page_size: u64) -> AppResult<PagedResponse<UserDto>> {
        let (total, items) = self.repo.list(page, page_size).await?;
        let items = items
            .iter()
            .map(|item| UserDto::from(item.to_owned()))
            .collect();
        Ok(PagedResponse::new(items, total, page, page_size))
    }
}
