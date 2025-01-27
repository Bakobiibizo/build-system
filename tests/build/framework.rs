use std::path::PathBuf;
use build_system::build::BuildManager;
use build_system::build::error::BuildError;
use build_system::state::StateManager;
use build_system::state::types::{TaskId, TaskState, TaskStatus, TaskMetadata};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

pub struct TestContext {
    pub state_manager: StateManager,
    pub build_manager: BuildManager,
}

impl TestContext {
    pub fn new() -> Self {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager.clone(), PathBuf::from("/tmp"));
        Self {
            state_manager,
            build_manager,
        }
    }

    pub async fn create_test_task(&self, task_id: &TaskId) -> Result<(), BuildError> {
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

        self.state_manager.create_task(task).await.map_err(BuildError::StateError)
    }

    pub async fn verify_task_status(&self, task_id: &TaskId, expected_status: TaskStatus) -> Result<(), BuildError> {
        let task = self.state_manager.get_task(task_id).await.map_err(BuildError::StateError)?;
        assert_eq!(task.status, expected_status);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_task() -> Result<(), BuildError> {
        let ctx = TestContext::new();
        let task_id = TaskId::new("test-task");
        ctx.create_test_task(&task_id).await?;

        let task = ctx.state_manager.get_task(&task_id).await.map_err(BuildError::StateError)?;
        assert_eq!(task.id.0, "test-task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.metadata.name, "echo test");
        assert_eq!(task.metadata.dependencies.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_setup_task_with_dependencies() -> Result<(), BuildError> {
        let ctx = TestContext::new();
        let dep1 = TaskId::new("dep-1");
        let dep2 = TaskId::new("dep-2");
        ctx.create_test_task(&dep1).await?;
        ctx.create_test_task(&dep2).await?;

        Ok(())
    }
}
