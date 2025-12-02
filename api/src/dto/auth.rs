use crate::dto::CreateUserDto;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 64))]
    #[schema(example = "some")]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

impl From<RegisterRequest> for CreateUserDto {
    fn from(value: RegisterRequest) -> Self {
        Self {
            username: value.username,
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[validate(schema(function = "validate_login_request"))]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 64))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8))]
    pub password: String,
}

fn validate_login_request(req: &LoginRequest) -> Result<(), ValidationError> {
    match (&req.username, &req.email) {
        (Some(_), None) | (None, Some(_)) => Ok(()),
        (Some(_), Some(_)) => Err(ValidationError::new("multiple_identifiers")),
        _ => Err(ValidationError::new("missing_identifier")),
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RefreshRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}
