use std::path::PathBuf;

use async_trait::async_trait;
use ras_errors::AppError;

use crate::application::csv_normalize::normalize_csv;
use crate::application::filename_validator::parse_filename;
use crate::domain::file::{FileExtension, FileSystemFile};
use crate::domain::repository::{FileSummary, FileSystemPort, FileSystemState};

#[derive(Debug)]
pub struct LocalFileSystem {
    root: PathBuf,
}

impl LocalFileSystem {
    pub fn new(root: PathBuf) -> Result<Self, AppError> {
        std::fs::create_dir_all(&root)
            .map_err(|e| AppError::InternalError(format!("mkdir: {e}")))?;
        Ok(Self { root })
    }

    fn path_for(&self, name: &str) -> Result<PathBuf, AppError> {
        let _ = parse_filename(name)?;
        Ok(self.root.join(name))
    }
}

#[async_trait]
impl FileSystemPort for LocalFileSystem {
    async fn read(&self, name: &str) -> Result<FileSystemFile, AppError> {
        let (_, ext) = parse_filename(name)?;
        let path = self.path_for(name)?;
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| AppError::NotFound(format!("read {name}: {e}")))?;
        Ok(FileSystemFile { name: name.into(), extension: ext, bytes })
    }

    async fn write(&self, file: FileSystemFile) -> Result<(), AppError> {
        let path = self.path_for(&file.name)?;
        let mut bytes = file.bytes;
        if file.extension == FileExtension::Csv {
            let s = String::from_utf8_lossy(&bytes).to_string();
            bytes = normalize_csv(&s).into_bytes();
        }
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| AppError::InternalError(format!("write {}: {e}", file.name)))?;
        Ok(())
    }

    async fn append(&self, name: &str, content: &str) -> Result<(), AppError> {
        let path = self.path_for(name)?;
        let mut current = tokio::fs::read(&path).await.unwrap_or_default();
        current.extend_from_slice(content.as_bytes());
        tokio::fs::write(&path, &current)
            .await
            .map_err(|e| AppError::InternalError(format!("append {name}: {e}")))?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<FileSummary>, AppError> {
        let mut out = Vec::new();
        let mut rd = tokio::fs::read_dir(&self.root)
            .await
            .map_err(|e| AppError::InternalError(format!("read_dir: {e}")))?;
        while let Some(entry) = rd
            .next_entry()
            .await
            .map_err(|e| AppError::InternalError(format!("read_dir entry: {e}")))?
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok((_, ext)) = parse_filename(&name) else {
                continue;
            };
            let meta = entry
                .metadata()
                .await
                .map_err(|e| AppError::InternalError(format!("metadata: {e}")))?;
            out.push(FileSummary { name, extension: ext, size_bytes: meta.len() });
        }
        Ok(out)
    }

    async fn snapshot(&self) -> Result<FileSystemState, AppError> {
        Ok(FileSystemState { root: self.root.clone(), files: self.list().await? })
    }
}
