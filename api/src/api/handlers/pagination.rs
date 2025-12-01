use crate::error::AppError;
use serde::Deserialize;
use std::num::NonZeroU64;

pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PAGE_SIZE: u64 = 20;
pub const MAX_PAGE_SIZE: u64 = 100;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: u64,
    pub page_size: NonZeroU64,
}

impl PaginationQuery {
    pub fn into_pagination(self) -> Result<Pagination, AppError> {
        if self.page == 0 {
            return Err(AppError::BadRequest("page must be >= 1".to_string()));
        }

        let page_size = NonZeroU64::new(self.page_size)
            .ok_or_else(|| AppError::BadRequest("page_size must be greater than 0".to_string()))?;

        if page_size.get() > MAX_PAGE_SIZE {
            return Err(AppError::BadRequest(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        Ok(Pagination {
            page: self.page,
            page_size,
        })
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_page_size() -> u64 {
    DEFAULT_PAGE_SIZE
}
