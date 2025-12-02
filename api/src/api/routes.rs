use crate::AppState;
use crate::api::{handlers, middleware};
use crate::error::ErrorResponse;
use axum::http::StatusCode;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};

pub fn create_router(state: Arc<AppState>) -> Router {
    let mut router = Router::new();

    router = router
        .route("/health", get(health_check))
        .nest("/api", Router::new().nest("/user", user_routes(state.clone())));

    router = router
        .fallback(handler_404)
        .layer(from_fn(middleware::error_handler));

    router = router
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(state.config.server.request_timeout_secs),
        ));

    router = middleware::trace(router);

    router.with_state(state)
}

fn user_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(handlers::user::create).get(handlers::user::list))
        .route("/{id}", get(handlers::user::find_by_id))
        .layer(from_fn_with_state(state, middleware::require_auth))
}

async fn health_check() -> Json<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("health".to_string(), "ok".to_string());
    Json(map)
}

async fn handler_404() -> (StatusCode, impl IntoResponse) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("nothing to see here".to_string())),
    )
}
