use serde::{Deserialize, Serialize};

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

impl PageParams {
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.page_size
    }
}

#[derive(Debug, Serialize)]
pub struct PageResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub has_more: bool,
}

impl<T: Serialize> PageResult<T> {
    pub fn new(items: Vec<T>, total: u64, params: &PageParams) -> Self {
        let has_more = params.page * params.page_size < total;
        Self {
            items,
            total,
            page: params.page,
            page_size: params.page_size,
            has_more,
        }
    }
}
