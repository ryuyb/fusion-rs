use crate::error::db::map_db_error;
use anyhow::Error as AnyhowError;
use sea_orm::DbErr;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal server error: {source}")]
    InternalServerError {
        #[source]
        source: AnyhowError,
    },

    #[error("BadRequest: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Record not found: {entity} with {field}={value}")]
    NotFound {
        entity: String,
        field: String,
        value: String,
    },

    #[error("Duplicate entry: {entity} with {field}={value}")]
    Duplicate {
        entity: String,
        field: String,
        value: String,
    },
}

impl From<AnyhowError> for AppError {
    fn from(source: AnyhowError) -> Self {
        Self::InternalServerError { source }
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        map_db_error(e)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        Self::BadRequest(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
