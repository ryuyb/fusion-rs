use crate::AppState;
use crate::dto::{CreateUserDto, UserDto};
use crate::error::AppResult;
use axum::Json;
use axum::extract::{Path, State};
use std::sync::Arc;

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateUserDto>,
) -> AppResult<Json<UserDto>> {
    state.services.user.create(data).await.map(Json)
}

pub async fn find_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserDto>> {
    state.services.user.find_by_id(id).await.map(Json)
}
