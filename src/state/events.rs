use serde::{Serialize, Deserialize};
use thiserror::Error;
use crate::state::types::{TaskId, TaskStatus};
use crate::state::manager::StateManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEvent {
    TaskCreated(TaskId),
    TaskUpdated(TaskId, TaskStatus),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),
    DependencyResolved(TaskId, Vec<TaskId>),
    SnapshotCreated(u32),
}

#[async_trait::async_trait]
pub trait StateEventHandler {
    async fn handle_event(&mut self, event: StateEvent) -> Result<(), StateEventError>;
}

#[derive(Debug, Error)]
pub enum StateEventError {
    #[error("Event processing failed: {0}")]
    ProcessingError(String),
    
    #[error("Event handler timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
    
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
}

#[async_trait::async_trait]
impl<T: StateManager + Send + Sync> StateEventHandler for T {
    async fn handle_event(&mut self, event: StateEvent) -> Result<(), StateEventError> {
        match event {
            StateEvent::TaskCreated(task_id) => {
                // Log task creation
                tracing::info!("Task created: {}", task_id);
                Ok(())
            },
            StateEvent::TaskUpdated(task_id, status) => {
                // Update task status
                tracing::info!("Task {} updated to status: {}", task_id, status);
                Ok(())
            },
            StateEvent::TaskCompleted(task_id) => {
                // Handle task completion
                tracing::info!("Task completed: {}", task_id);
                Ok(())
            },
            StateEvent::TaskFailed(task_id, error) => {
                // Handle task failure
                tracing::error!("Task {} failed: {}", task_id, error);
                Ok(())
            },
            StateEvent::DependencyResolved(task_id, dependencies) => {
                // Log dependency resolution
                tracing::info!("Dependencies for task {} resolved: {:?}", task_id, dependencies);
                Ok(())
            },
            StateEvent::SnapshotCreated(version) => {
                // Log snapshot creation
                tracing::info!("State snapshot created: version {}", version);
                Ok(())
            },
        }
    }
}
