use thiserror::Error;
use std::io;
use serde_json;

#[derive(Error, Debug)]
pub enum DocumentationError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Document not found")]
    DocumentNotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Unknown documentation error: {0}")]
    Other(String),
}

impl From<String> for DocumentationError {
    fn from(message: String) -> Self {
        DocumentationError::Other(message)
    }
}

impl From<&str> for DocumentationError {
    fn from(message: &str) -> Self {
        DocumentationError::Other(message.to_string())
    }
}
