use crate::api::{handlers, middleware};
use crate::AppState;
use axum::http::StatusCode;
use axum::routing::get;
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
        .nest("/api", Router::new().nest("/user", user_routes()));

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

fn user_routes() -> Router<Arc<AppState>> {
    Router::new().route("/{id}", get(handlers::user::find_by_id))
}

async fn health_check() -> Json<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.insert("health".to_string(), "ok".to_string());
    Json(map)
}
