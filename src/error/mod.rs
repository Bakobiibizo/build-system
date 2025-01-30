use thiserror::Error;

/// Centralized error handling for the build system
#[derive(Debug, Error)]
pub enum BuildSystemError {
    /// Errors related to project generation
    #[error("Project generation failed: {0}")]
    ProjectGenerationError(String),

    /// Errors related to configuration
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Errors related to file system operations
    #[error("File system error: {0}")]
    FileSystemError(String),

    /// Errors related to template processing
    #[error("Template processing error: {0}")]
    TemplateError(String),

    /// Catch-all for other unexpected errors
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Result type using the BuildSystemError
pub type Result<T> = std::result::Result<T, BuildSystemError>;
