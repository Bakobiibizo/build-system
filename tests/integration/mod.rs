use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use build_system::state::types::{
    TaskId, 
    TaskState, 
    TaskMetadata, 
    TaskStatus,
    TaskPriority,
};
use build_system::build::error::BuildError;

pub trait StateManagerProcessor {
    async fn update_task_state(&self, task_id: TaskId, state: TaskState) -> Result<(), BuildError>;
    async fn get_task_state(&self, task_id: TaskId) -> Result<TaskState, BuildError>;
}

pub struct MockStateManagerProcessor;

impl StateManagerProcessor for MockStateManagerProcessor {
    async fn update_task_state(&self, _task_id: TaskId, _state: TaskState) -> Result<(), BuildError> {
        Ok(())
    }

    async fn get_task_state(&self, _task_id: TaskId) -> Result<TaskState, BuildError> {
        Ok(create_test_task_state())
    }
}

fn create_test_task_metadata() -> TaskMetadata {
    TaskMetadata {
        name: "Test Task".to_string(),
        description: Some("A test task for integration testing".to_string()),
        owner: "test-user".to_string(),
        priority: TaskPriority::High,
        tags: vec!["test".to_string(), "integration".to_string()],
        estimated_duration: Duration::from_micros(1),
        dependencies: vec![],
        additional_info: HashMap::new(),
    }
}

fn create_test_task_state() -> TaskState {
    TaskState {
        id: TaskId::new("test-1"),
        status: TaskStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: create_test_task_metadata(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_state_update() {
        let processor = MockStateManagerProcessor;
        let task_id = TaskId::new("test-1");
        let state = create_test_task_state();
        
        let result = processor.update_task_state(task_id, state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_state_retrieval() {
        let processor = MockStateManagerProcessor;
        let task_id = TaskId::new("test-1");
        
        let result = processor.get_task_state(task_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let processor = MockStateManagerProcessor;
        let task_id = TaskId::new("test-1");
        let mut state = create_test_task_state();
        
        // Update task state
        processor.update_task_state(task_id.clone(), state.clone()).await?;
        
        // Retrieve and verify
        state = processor.get_task_state(task_id).await?;
        assert_eq!(state.status, TaskStatus::Pending);
        
        Ok(())
    }
}
