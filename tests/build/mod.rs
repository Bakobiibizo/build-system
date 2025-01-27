use std::path::PathBuf;
use build_system::build::BuildManager;
use build_system::build::error::BuildError;
use build_system::state::StateManager;
use build_system::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

mod framework;
use framework::TestContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_manager() -> Result<(), BuildError> {
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

        Ok(())
    }

    #[tokio::test]
    async fn test_build_manager_invalid_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager, PathBuf::from("/tmp"));

        let invalid_task_id = TaskId::new("non-existent-task");
        assert!(build_manager.execute_task(&invalid_task_id).await.is_err());
        Ok(())
    }
}
