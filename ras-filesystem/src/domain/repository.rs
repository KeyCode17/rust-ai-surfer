use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::file::{FileExtension, FileSystemFile};

#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("invalid filename: {0}")]
    InvalidFilename(String),
    #[error("io: {0}")]
    Io(String),
}

impl From<FileSystemError> for AppError {
    fn from(e: FileSystemError) -> Self {
        Self::ValidationError(e.to_string())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileSystemState {
    pub root: PathBuf,
    pub files: Vec<FileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub name: String,
    pub extension: FileExtension,
    pub size_bytes: u64,
}

#[async_trait]
pub trait FileSystemPort: Send + Sync + 'static {
    async fn read(&self, name: &str) -> Result<FileSystemFile, AppError>;
    async fn write(&self, file: FileSystemFile) -> Result<(), AppError>;
    async fn append(&self, name: &str, content: &str) -> Result<(), AppError>;
    async fn list(&self) -> Result<Vec<FileSummary>, AppError>;
    async fn snapshot(&self) -> Result<FileSystemState, AppError>;
}
