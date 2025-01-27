#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use crate::state::error::StateError;
    use crate::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata};
    use crate::state::StateManager;

    #[tokio::test]
    async fn test_task_state() {
        let id = "test-task-1";
        let task_id = TaskId::new(id);
        let task = TaskState::new(task_id.clone());

        assert_eq!(task_id.to_string(), id);
        assert_eq!(TaskId::new(id), task_id);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_task_metadata() {
        let id = "test-task-1";
        let task_id = TaskId::new(id);
        let mut task = TaskState::new(task_id);

        // Update metadata
        task.metadata = TaskMetadata {
            dependencies: vec![],
            estimated_duration: Duration::from_secs(60),
            priority: 1,
            tags: vec!["test".to_string()],
            name: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            owner: "test-owner".to_string(),
            additional_info: HashMap::new(),
        };

        assert_eq!(task.metadata.priority, 1);
        assert_eq!(task.metadata.tags.len(), 1);
        assert_eq!(task.metadata.tags[0], "test");
    }

    #[tokio::test]
    async fn test_create_task() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id = TaskId::new("test1");
        let mut task = TaskState::new(task_id.clone());
        task.metadata.name = "test".to_string();
        
        state_manager.create_task(task).await?;
        let retrieved_task = state_manager.get_task(&task_id).await?;
        assert_eq!(retrieved_task.id, task_id);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_update_task_status() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id = TaskId::new("test1");
        let mut task = TaskState::new(task_id.clone());
        task.metadata.name = "test".to_string();
        
        state_manager.create_task(task).await?;
        state_manager.update_task_status(&task_id, TaskStatus::Running).await?;
        
        let updated_task = state_manager.get_task(&task_id).await?;
        assert_eq!(updated_task.status, TaskStatus::Running);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_task() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id = TaskId::new("test1");
        let mut task = TaskState::new(task_id.clone());
        task.metadata.name = "test".to_string();
        
        state_manager.create_task(task).await?;
        state_manager.delete_task(&task_id).await?;
        
        let result = state_manager.get_task(&task_id).await;
        assert!(result.is_err());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id1 = TaskId::new("test1");
        let task_id2 = TaskId::new("test2");
        
        let mut task1 = TaskState::new(task_id1.clone());
        task1.metadata.name = "test1".to_string();
        
        let mut task2 = TaskState::new(task_id2.clone());
        task2.metadata.name = "test2".to_string();
        
        state_manager.create_task(task1).await?;
        state_manager.create_task(task2).await?;
        
        let tasks = state_manager.list_tasks().await?;
        assert_eq!(tasks.len(), 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_tasks_by_status() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id1 = TaskId::new("test1");
        let task_id2 = TaskId::new("test2");
        
        let mut task1 = TaskState::new(task_id1.clone());
        task1.metadata.name = "test1".to_string();
        
        let mut task2 = TaskState::new(task_id2.clone());
        task2.metadata.name = "test2".to_string();
        
        state_manager.create_task(task1).await?;
        state_manager.create_task(task2).await?;
        
        state_manager.update_task_status(&task_id1, TaskStatus::Running).await?;
        
        let running_tasks = state_manager.get_tasks_by_status(TaskStatus::Running).await?;
        assert_eq!(running_tasks.len(), 1);
        assert_eq!(running_tasks[0].id, task_id1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_ready_tasks() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id1 = TaskId::new("test1");
        let task_id2 = TaskId::new("test2");
        
        let mut task1 = TaskState::new(task_id1.clone());
        task1.metadata.name = "test1".to_string();
        
        let mut task2 = TaskState::new(task_id2.clone());
        task2.metadata.name = "test2".to_string();
        
        state_manager.create_task(task1).await?;
        state_manager.create_task(task2).await?;
        
        let ready_tasks = state_manager.get_ready_tasks().await?;
        assert_eq!(ready_tasks.len(), 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_tasks_by_status_original() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id1 = TaskId::new("test1");
        let task_id2 = TaskId::new("test2");
        
        let mut task1 = TaskState::new(task_id1.clone());
        task1.metadata.name = "test1".to_string();
        
        let mut task2 = TaskState::new(task_id2.clone());
        task2.metadata.name = "test2".to_string();
        
        state_manager.create_task(task1).await?;
        state_manager.create_task(task2).await?;
        
        state_manager.update_task_status(&task_id1, TaskStatus::Running).await?;
        
        let running_tasks = state_manager.get_tasks_by_status(TaskStatus::Running).await?;
        assert_eq!(running_tasks.len(), 1);
        assert_eq!(running_tasks[0].id, task_id1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_ready_tasks_original() -> Result<(), StateError> {
        let state_manager = StateManager::new();
        let task_id1 = TaskId::new("test1");
        let task_id2 = TaskId::new("test2");
        
        let mut task1 = TaskState::new(task_id1.clone());
        task1.metadata.name = "test1".to_string();
        
        let mut task2 = TaskState::new(task_id2.clone());
        task2.metadata.name = "test2".to_string();
        
        state_manager.create_task(task1).await?;
        state_manager.create_task(task2).await?;
        
        let ready_tasks = state_manager.get_ready_tasks().await?;
        assert_eq!(ready_tasks.len(), 2);
        
        Ok(())
    }
}
