use crate::dto::UserDto;
use crate::error::AppResult;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

pub async fn find_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserDto>> {
    let user_dto = state.services.user.find_by_id(id).await?;
    Ok(Json(user_dto))
}
