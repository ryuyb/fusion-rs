use std::sync::Arc;
use axum::extract::Request;
use axum::http::HeaderName;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info_span};
use crate::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn trace(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    
    router.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(
                x_request_id.clone(),
                MakeRequestUuid,
            ))
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    // Log the request id as generated.
                    let request_id = request.headers().get(REQUEST_ID_HEADER);

                    match request_id {
                        Some(request_id) => info_span!(
                            "http_request",
                            method = display(request.method()),
                            uri = display(request.uri()),
                            version = debug(request.version()),
                            request_id = ?request_id,
                        ),
                        None => {
                            error!("could not extract request_id");
                            info_span!("http_request")
                        }
                    }
                }),
            )
            // send headers from request to response headers
            .layer(PropagateRequestIdLayer::new(x_request_id)),
    )
}
