use crate::AppState;
use crate::api::doc::{ApiDoc, HEALTH_TAG};
use crate::api::{handlers, middleware};
use crate::error::ErrorResponse;
use axum::http::StatusCode;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::IntoResponse;
use axum::{Json, Router};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: Arc<AppState>) -> Router {
    let (mut router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        .nest(
            "/api/v1",
            OpenApiRouter::new()
                .nest("/auth", auth_routes())
                .nest("/user", user_routes(state.clone())),
        )
        .split_for_parts();

    router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

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

fn user_routes(state: Arc<AppState>) -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(handlers::user::create))
        .routes(routes!(handlers::user::list))
        .routes(routes!(handlers::user::find_by_id))
        .layer(from_fn_with_state(state, middleware::require_auth))
}

fn auth_routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(handlers::auth::register))
        .routes(routes!(handlers::auth::login))
        .routes(routes!(handlers::auth::refresh))
}

#[utoipa::path(
    get,
    path = "/health",
    tag = HEALTH_TAG,
    responses(
         (status = 200, description = "Success", body = HashMap<String, String>)
    )
)]
async fn health_check() -> impl IntoResponse {
    let mut map = HashMap::new();
    map.insert("health".to_string(), "ok".to_string());
    (StatusCode::OK, Json(map))
}

async fn handler_404() -> (StatusCode, impl IntoResponse) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("nothing to see here".to_string())),
    )
}
