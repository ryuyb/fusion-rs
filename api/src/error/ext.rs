use crate::error::{AppError, AppResult};

pub trait IntoAppResult<T> {
    fn into_app_result(self) -> AppResult<T>;
}

impl<T, E> IntoAppResult<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn into_app_result(self) -> AppResult<T> {
        self.map_err(Into::into)
    }
}

pub trait Entity {
    const NAME: &'static str;

    fn not_found_by(field: &str, value: impl ToString) -> AppError {
        AppError::NotFound {
            entity: Self::NAME.to_string(),
            field: field.to_string(),
            value: value.to_string(),
        }
    }
}
