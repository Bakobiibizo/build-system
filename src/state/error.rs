use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),
    
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Dependencies not met: {0}")]
    DependenciesNotMet(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
