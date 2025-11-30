use crate::error::{AppError, AppResult};
use axum::Json;

pub mod user;

pub trait ToJsonResult {
    type Item;
    fn to_json_result(self) -> AppResult<Json<Self::Item>>;
}

impl<T, E> ToJsonResult for Result<T, E>
where
    E: Into<AppError>,
{
    type Item = T;

    fn to_json_result(self) -> AppResult<Json<T>> {
        self.map(Json).map_err(Into::into)
    }
}
