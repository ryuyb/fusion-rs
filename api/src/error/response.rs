use crate::error::types::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    pub timestamp: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self {
            message,
            details: None,
            timestamp: chrono::Utc::now().timestamp(),
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id.clone());
        self
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Duplicate { .. } => StatusCode::CONFLICT,
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let mut response = ErrorResponse::new(self.to_string());

        match self {
            Self::NotFound {
                entity,
                field,
                value,
            } => {
                response = response.with_details(json!({
                    "entity": entity,
                    "field": field,
                    "value": value,
                }))
            }
            Self::Duplicate {
                entity,
                field,
                value,
            } => {
                response = response.with_details(json!({
                    "entity": entity,
                    "field": field,
                    "value": value,
                }))
            }
            _ => {}
        }

        response
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.to_error_response());

        (status, body).into_response()
    }
}
