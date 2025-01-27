use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;

use build_system::build::BuildManager;
use build_system::build::error::BuildError;
use build_system::state::StateManager;
use build_system::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata};
use chrono::Utc;

pub trait StateManagerProcessor {
    async fn update_task_state(&self, task_id: TaskId, state: TaskState) -> Result<(), BuildError>;
}

pub struct MockStateManagerProcessor;

impl StateManagerProcessor for MockStateManagerProcessor {
    async fn update_task_state(&self, _task_id: TaskId, _state: TaskState) -> Result<(), BuildError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_flow() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager.clone(), PathBuf::from("/tmp"));

        // Create a test task
        let task_id = TaskId::new("test-task");
        let task = TaskState {
            id: task_id.clone(),
            status: TaskStatus::Pending,
            metadata: TaskMetadata {
                name: "echo test".to_string(),
                description: Some("A test task".to_string()),
                owner: "test".to_string(),
                dependencies: vec![],
                estimated_duration: Duration::from_secs(60),
                priority: 1,
                tags: vec!["test".to_string()],
                additional_info: HashMap::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        state_manager.create_task(task).await.map_err(BuildError::StateError)?;
        build_manager.execute_task(&task_id).await?;

        let updated_task = state_manager.get_task(&task_id).await.map_err(BuildError::StateError)?;
        assert_eq!(updated_task.status, TaskStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_workflow() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager.clone(), PathBuf::from("/tmp"));

        // Create a test task
        let task_id = TaskId::new("test-task");
        let task = TaskState {
            id: task_id.clone(),
            status: TaskStatus::Pending,
            metadata: TaskMetadata {
                name: "echo test".to_string(),
                description: Some("A test task".to_string()),
                owner: "test".to_string(),
                dependencies: vec![],
                estimated_duration: Duration::from_secs(60),
                priority: 1,
                tags: vec!["test".to_string()],
                additional_info: HashMap::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        state_manager.create_task(task.clone()).await.map_err(BuildError::StateError)?;
        build_manager.execute_task(&task_id).await?;

        let updated_task = state_manager.get_task(&task_id).await.map_err(BuildError::StateError)?;
        assert_eq!(updated_task.status, TaskStatus::Completed);

        Ok(())
    }
}
