use crate::AppState;
use crate::error::AppError;
use crate::utils::jwt::JwtClaims;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, header::AUTHORIZATION};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tracing::warn;

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub claims: JwtClaims,
}

impl AuthContext {
    pub fn user_id(&self) -> i32 {
        self.claims.user_id
    }
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    match authenticate(state, &mut request) {
        Ok(_) => next.run(request).await,
        Err(err) => err.into_response(),
    }
}

fn authenticate(state: Arc<AppState>, request: &mut Request) -> Result<(), AppError> {
    let token = extract_bearer_token(request.headers())?;
    let claims = state.jwt.decode_access_token(token).map_err(|err| {
        warn!(?err, "failed to decode access token");
        AppError::Unauthorized("Invalid or expired access token".into())
    })?;

    request.extensions_mut().insert(AuthContext { claims });
    Ok(())
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let header = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let header_str = header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".into()))?;

    let token = header_str
        .strip_prefix(BEARER_PREFIX)
        .ok_or_else(|| AppError::Unauthorized("Authorization header must be Bearer token".into()))?
        .trim();

    if token.is_empty() {
        return Err(AppError::Unauthorized(
            "Authorization header contains an empty token".into(),
        ));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn extracts_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer abc.def.ghi"),
        );

        let token = extract_bearer_token(&headers).expect("token should be extracted");
        assert_eq!(token, "abc.def.ghi");
    }

    #[test]
    fn rejects_missing_header() {
        let headers = HeaderMap::new();
        let err = extract_bearer_token(&headers).expect_err("should fail");
        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[test]
    fn rejects_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic abc"));

        let err = extract_bearer_token(&headers).expect_err("should fail");
        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
