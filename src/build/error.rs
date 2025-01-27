use thiserror::Error;
use crate::state::error::StateError;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("State error: {0}")]
    StateError(#[from] StateError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
