use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleResponse<T> {
    pub data: T,
    pub message: String,
}

impl<T> SingleResponse<T> {
    pub fn new(data: T, message: impl Into<String>) -> Self {
        Self { data, message: message.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}
