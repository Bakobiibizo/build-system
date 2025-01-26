use thiserror::Error;
use crate::state::StateError;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Insufficient resources available")]
    InsufficientResources,

    #[error("Task cancelled: {0}")]
    TaskCancelled(String),

    #[error("Invalid task state: {0}")]
    InvalidTaskState(String),

    #[error("Dependencies not met for task: {0}")]
    DependenciesNotMet(String),

    #[error("Task execution failed: {0}")]
    TaskExecutionError(String),

    #[error("Task timed out: {0}")]
    TimeoutError(String),

    #[error("State error: {0}")]
    StateError(#[from] StateError),
}
