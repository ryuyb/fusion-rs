use crate::error::Entity;
use entity::user::Model;

impl Entity for Model {
    const NAME: &'static str = "user";
}
