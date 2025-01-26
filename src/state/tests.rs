#[cfg(test)]
mod tests {
    use crate::state::{StateManager, StateError};
    use crate::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata, TaskPriority};
    use std::collections::HashMap;
    use std::time::Duration;
    use chrono::Utc;

    #[tokio::test]
    async fn test_state_operations() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = create_test_task("test-1");

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

    fn create_test_task(id: &str) -> TaskState {
        TaskState {
            id: TaskId::new(id),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: TaskMetadata {
                name: format!("Test Task {}", id),
                description: Some("Test task description".to_string()),
                owner: "test-owner".to_string(),
                priority: TaskPriority::Medium,
                tags: vec!["test".to_string()],
                estimated_duration: Duration::from_micros(1), // Use microsecond duration for tests
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        }
    }
}
