use crate::AppState;
use crate::api::ValidatedJson;
use crate::dto::{LoginRequest, RefreshRequest, RegisterRequest};
use crate::error::{AppError, AppResult};
use crate::service::{AuthTokens, LoginIdentifier};
use axum::Json;
use axum::extract::State;
use std::sync::Arc;
use crate::api::doc::AUTH_TAG;

#[utoipa::path(
    post,
    path = "/register",
    tag = AUTH_TAG,
    request_body = RegisterRequest,
    responses(
         (status = 200, description = "Register an new user", body = AuthTokens)
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> AppResult<Json<AuthTokens>> {
    state.services.auth.register(payload).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/login",
    tag = AUTH_TAG,
    request_body = LoginRequest,
    responses(
         (status = 200, description = "Login with username or email", body = AuthTokens)
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<Json<AuthTokens>> {
    let identifier = resolve_identifier(&payload)?;
    state
        .services
        .auth
        .authenticate(identifier, &payload.password)
        .await
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/refresh",
    tag = AUTH_TAG,
    request_body = RefreshRequest,
    responses(
         (status = 200, description = "Refresh access token by refresh token", body = AuthTokens)
    )
)]
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<RefreshRequest>,
) -> AppResult<Json<AuthTokens>> {
    state
        .services
        .auth
        .refresh_tokens(&payload.refresh_token)
        .await
        .map(Json)
}

fn resolve_identifier<'a>(payload: &'a LoginRequest) -> AppResult<LoginIdentifier<'a>> {
    match (payload.username.as_deref(), payload.email.as_deref()) {
        (Some(username), None) => Ok(LoginIdentifier::Username(username)),
        (None, Some(email)) => Ok(LoginIdentifier::Email(email)),
        _ => Err(AppError::BadRequest(
            "A username or email is required to login".into(),
        )),
    }
}
