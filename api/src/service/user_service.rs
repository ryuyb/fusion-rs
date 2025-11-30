use crate::dto::UserDto;
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

    pub async fn find_by_id(&self, id: i32) -> AppResult<UserDto> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Model::not_found_by("id", id))
            .map(|model| UserDto::from(model))
            .into_app_result()
    }
}
