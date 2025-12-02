mod auth;
mod user;

pub use auth::*;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use utoipa::ToSchema;
pub use user::*;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> PagedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: NonZeroU64) -> Self {
        let page_size_value = page_size.get();
        let total_pages = (total + page_size_value - 1) / page_size_value;

        Self {
            items,
            total,
            page,
            page_size: page_size_value,
            total_pages,
        }
    }
}
