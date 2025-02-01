use thiserror::Error;

/// Custom error types for prompt-related operations
#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Prompt validation failed: {0}")]
    ValidationError(String),

    #[error("Prompt storage error: {0}")]
    StorageError(String),

    #[error("Prompt not found")]
    NotFound,
}
