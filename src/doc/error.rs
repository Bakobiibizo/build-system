use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid document type: {0}")]
    InvalidDocType(String),

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Document already exists: {0}")]
    AlreadyExists(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
