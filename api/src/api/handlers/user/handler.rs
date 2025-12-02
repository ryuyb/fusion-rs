use crate::api::doc::{AUTH_TAG, USER_TAG};
use crate::api::handlers::pagination::{Pagination, PaginationQuery};
use crate::api::middleware::AuthContext;
use crate::dto::{CreateUserDto, PagedResponse, UserDto};
use crate::error::AppResult;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/",
    tag = USER_TAG,
    responses(
         (status = 200, description = "Create a new user", body = UserDto)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateUserDto>,
) -> AppResult<Json<UserDto>> {
    state.services.user.create(data).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/{id}",
    tag = USER_TAG,
    responses(
         (status = 200, description = "Find user by id", body = UserDto)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn find_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(auth): Extension<AuthContext>,
) -> AppResult<Json<UserDto>> {
    tracing::debug!(user_id = auth.user_id(), "fetching user by id");
    state.services.user.find_by_id(id).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/",
    tag = USER_TAG,
    responses(
         (status = 200, description = "List users by page", body = PagedResponse<UserDto>)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationQuery>,
    Extension(auth): Extension<AuthContext>,
) -> AppResult<Json<PagedResponse<UserDto>>> {
    tracing::debug!(user_id = auth.user_id(), "listing users");
    let Pagination { page, page_size } = params.into_pagination()?;
    state.services.user.list(page, page_size).await.map(Json)
}
