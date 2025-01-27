use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use tempfile::TempDir;

use build_system::state::types::{
    TaskId,
    TaskState,
    TaskStatus,
    TaskMetadata,
    StateSnapshot,
};
use build_system::state::StateManager;
use build_system::state::error::StateError;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_state() {
        let id = "test-task-1";
        let task_id = TaskId::new(id);
        let mut task = TaskState::new(task_id.clone());

        // Update metadata
        task.metadata = TaskMetadata {
            name: format!("Test Task {}", id),
            description: Some("Test task description".to_string()),
            owner: "test-owner".to_string(),
            dependencies: vec![],
            estimated_duration: Duration::from_secs(60),
            priority: 1,
            tags: vec!["test".to_string()],
            additional_info: HashMap::new(),
        };

        assert_eq!(task.id, task_id);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.metadata.name, format!("Test Task {}", id));
        assert_eq!(task.metadata.priority, 1);
        assert_eq!(task.metadata.tags.len(), 1);
        assert_eq!(task.metadata.tags[0], "test");
    }

    #[tokio::test]
    async fn test_task_metadata() {
        let id = "test-task-1";
        let task_id = TaskId::new(id);
        let mut task = TaskState::new(task_id);

        // Update metadata
        task.metadata = TaskMetadata {
            name: "Test Task test-1".to_string(),
            description: Some("Test task description".to_string()),
            owner: "test-owner".to_string(),
            dependencies: vec![],
            estimated_duration: Duration::from_secs(60),
            priority: 1,
            tags: vec!["test".to_string()],
            additional_info: HashMap::new(),
        };

        assert_eq!(task.metadata.name, "Test Task test-1");
        assert_eq!(task.metadata.priority, 1);
        assert_eq!(task.metadata.tags.len(), 1);
        assert_eq!(task.metadata.tags[0], "test");
    }

    #[tokio::test]
    async fn test_state_manager_create_task() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task = TaskState {
            id: TaskId::new("test-1"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-1".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        state_manager.create_task(task.clone()).await?;
        let retrieved_task = state_manager.get_task(&task.id).await?;

        assert_eq!(retrieved_task.id, task.id);
        assert_eq!(retrieved_task.metadata.name, task.metadata.name);
        assert_eq!(retrieved_task.status, TaskStatus::Pending);

        Ok(())
    }

    #[tokio::test]
    async fn test_state_manager_update_task() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let mut task = TaskState {
            id: TaskId::new("test-2"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-2".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        // Create initial task
        state_manager.create_task(task.clone()).await?;

        // Update task status
        task.status = TaskStatus::Running;
        state_manager.update_task_status(&task.id, TaskStatus::Running).await?;

        let updated_task = state_manager.get_task(&task.id).await?;
        assert_eq!(updated_task.status, TaskStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_state_manager_dependencies() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task1 = TaskState {
            id: TaskId::new("test-3"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-3".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };
        let mut task2 = TaskState {
            id: TaskId::new("test-4"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-4".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        // Add task1 as dependency for task2
        task2.metadata.dependencies = vec![task1.id.clone()];

        // Create both tasks
        state_manager.create_task(task1.clone()).await?;
        state_manager.create_task(task2.clone()).await?;

        let retrieved_task2 = state_manager.get_task(&task2.id).await?;
        assert!(retrieved_task2.metadata.dependencies.contains(&task1.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_state_manager_delete_task() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task = TaskState {
            id: TaskId::new("test-5"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-5".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        // Create and then delete task
        state_manager.create_task(task.clone()).await?;
        state_manager.delete_task(&task.id).await?;

        // Verify task is deleted
        let result = state_manager.get_task(&task.id).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_task_status() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = TaskState {
            id: TaskId::new("test-2"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-2".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        manager.create_task(task.clone()).await?;
        manager.update_task_status(&task.id, TaskStatus::Running).await?;

        let stored_task = manager.get_task(&task.id).await?;
        assert_eq!(stored_task.status, TaskStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_task() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = TaskState {
            id: TaskId::new("test-3"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-3".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        manager.create_task(task.clone()).await?;
        manager.delete_task(&task.id).await?;

        let result = manager.get_task(&task.id).await;
        assert!(matches!(result, Err(StateError::TaskNotFound(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task1 = TaskState {
            id: TaskId::new("test-4"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-4".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };
        let task2 = TaskState {
            id: TaskId::new("test-5"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-5".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        manager.create_task(task1.clone()).await?;
        manager.create_task(task2.clone()).await?;

        let tasks = manager.list_tasks().await?;
        assert_eq!(tasks.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_state_operations() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = TaskState {
            id: TaskId::new("test-1"),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: "Test Task test-1".to_string(),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: 1,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        };

        // Test create and get
        manager.create_task(task.clone()).await?;
        let retrieved = manager.get_task(&task.id).await?;
        assert_eq!(retrieved, task);

        // Test update status
        manager.update_task_status(&task.id, TaskStatus::Running).await?;
        let updated = manager.get_task(&task.id).await?;
        assert_eq!(updated.status, TaskStatus::Running);

        // Test list tasks
        let tasks = manager.list_tasks().await?;
        assert_eq!(tasks.len(), 1);

        // Test delete
        manager.delete_task(&task.id).await?;
        assert!(matches!(
            manager.get_task(&task.id).await,
            Err(StateError::TaskNotFound(_))
        ));

        Ok(())
    }
}
