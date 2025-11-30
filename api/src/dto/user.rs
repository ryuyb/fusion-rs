use serde::{Deserialize, Serialize};
use entity::user::Model;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i32,
}

impl From<Model> for UserDto {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
        }
    }
}
