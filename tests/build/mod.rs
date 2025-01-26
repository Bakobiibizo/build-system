use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;

use build_system::build::BuildEngine;
use build_system::build::error::BuildError;
use build_system::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata, TaskPriority};
use build_system::state::StateManager;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_build_task() -> TaskState {
        TaskState {
            id: TaskId::new("test-build-task"),
            metadata: TaskMetadata {
                name: "test-build-task".to_string(),
                description: Some("Test build task".to_string()),
                dependencies: vec![],
                priority: TaskPriority::High,
                owner: "test-user".to_string(),
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1), // Use microsecond duration for tests
                additional_info: HashMap::new(),
            },
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_execute_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        let task = create_test_build_task();

        // Add task to state manager
        state_manager.create_task(task.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        // Execute task
        engine.execute_task(&task.id).await?;

        // Verify final state
        let status = engine.get_task_status(&task.id).await?;
        assert_eq!(status, TaskStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_state_transitions() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        let task = create_test_build_task();

        // Add task to state manager
        state_manager.create_task(task.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        // Verify initial state
        let status = engine.get_task_status(&task.id).await?;
        assert_eq!(status, TaskStatus::Pending);

        // Start execution
        engine.execute_task(&task.id).await?;

        // Verify final state
        let status = engine.get_task_status(&task.id).await?;
        assert_eq!(status, TaskStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_status() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        let task = create_test_build_task();

        // Add task to state manager
        state_manager.create_task(task.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        let status = engine.get_task_status(&task.id).await?;
        assert_eq!(status, TaskStatus::Pending);

        Ok(())
    }
}
