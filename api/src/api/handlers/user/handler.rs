use crate::AppState;
use crate::api::handlers::pagination::{Pagination, PaginationQuery};
use crate::dto::{CreateUserDto, PagedResponse, UserDto};
use crate::error::AppResult;
use axum::Json;
use axum::extract::{Path, Query, State};
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

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationQuery>,
) -> AppResult<Json<PagedResponse<UserDto>>> {
    let Pagination { page, page_size } = params.into_pagination()?;
    state.services.user.list(page, page_size).await.map(Json)
}
