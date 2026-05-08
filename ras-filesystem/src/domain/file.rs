use ras_errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileExtension {
    Md,
    Json,
    Jsonl,
    Csv,
    Html,
    Docx,
    Pdf,
    Txt,
}

impl FileExtension {
    pub fn parse(ext: &str) -> Result<Self, AppError> {
        match ext.to_ascii_lowercase().as_str() {
            "md" => Ok(Self::Md),
            "json" => Ok(Self::Json),
            "jsonl" => Ok(Self::Jsonl),
            "csv" => Ok(Self::Csv),
            "html" | "htm" => Ok(Self::Html),
            "docx" => Ok(Self::Docx),
            "pdf" => Ok(Self::Pdf),
            "txt" => Ok(Self::Txt),
            other => Err(AppError::BadRequest(format!(
                "unsupported extension: {other}"
            ))),
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Csv => "csv",
            Self::Html => "html",
            Self::Docx => "docx",
            Self::Pdf => "pdf",
            Self::Txt => "txt",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemFile {
    pub name: String,
    pub extension: FileExtension,
    pub bytes: Vec<u8>,
}

pub trait BaseFile: Send + Sync + 'static {
    fn extension(&self) -> FileExtension;
    fn allowed_extensions() -> &'static [FileExtension]
    where
        Self: Sized;
    fn read(&self, raw: &[u8]) -> Result<String, AppError>;
    fn write(&self, content: &str) -> Result<Vec<u8>, AppError>;
}
