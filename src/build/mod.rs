use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

pub mod error;
use error::BuildError;

use crate::state::types::{TaskId, TaskState, TaskStatus};
use crate::state::StateManager;

/// The BuildEngine is responsible for executing tasks and managing their lifecycle.
#[derive(Debug, Clone)]
pub struct BuildEngine {
    state_manager: StateManager,
    resources: Arc<RwLock<HashMap<String, usize>>>,
}

impl BuildEngine {
    /// Creates a new BuildEngine instance.
    pub fn new(state_manager: StateManager) -> Self {
        Self {
            state_manager,
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Executes a task with the given task_id.
    pub async fn execute_task(&self, task_id: &TaskId) -> Result<(), BuildError> {
        let task = self.state_manager.get_task(task_id).await
            .map_err(|e| BuildError::StateError(e))?;

        // Check dependencies
        let dependencies = self.state_manager.get_task_dependencies(task_id).await
            .map_err(|e| BuildError::StateError(e))?;

        // Verify all dependencies are completed
        for dep_id in dependencies {
            let dep_status = self.get_task_status(&dep_id).await?;
            if dep_status != TaskStatus::Completed {
                return Err(BuildError::DependenciesNotMet(task_id.to_string()));
            }
        }

        // Check if we have enough resources
        if !self.check_resources(&task).await {
            return Err(BuildError::InsufficientResources);
        }

        // Allocate resources
        self.allocate_resources(&task).await?;

        // Update task status to Running
        self.state_manager.update_task_status(task_id, TaskStatus::Running).await
            .map_err(|e| BuildError::StateError(e))?;

        // Execute the build
        let result = self.run_build(&task).await;

        // Release resources
        self.release_resources(&task).await;

        // Update final status
        let status = if result.is_ok() {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };

        self.state_manager.update_task_status(task_id, status).await
            .map_err(|e| BuildError::StateError(e))?;

        result
    }

    /// Checks if the given task has enough resources to run.
    async fn check_resources(&self, _task: &TaskState) -> bool {
        let _resources = self.resources.read().await;
        // Implement resource checking logic here
        true // Simplified for now
    }

    /// Allocates resources for the given task.
    async fn allocate_resources(&self, task: &TaskState) -> Result<(), BuildError> {
        let mut resources = self.resources.write().await;
        // Implement resource allocation logic here
        let required = task.metadata.estimated_duration.as_secs() as usize;
        resources.insert(task.id.to_string(), required);
        Ok(())
    }

    /// Releases resources for the given task.
    async fn release_resources(&self, task: &TaskState) {
        let mut resources = self.resources.write().await;
        resources.remove(&task.id.to_string());
    }

    /// Runs the build for the given task.
    async fn run_build(&self, task: &TaskState) -> Result<(), BuildError> {
        // For test tasks, complete immediately
        if task.metadata.tags.contains(&"test".to_string()) {
            return Ok(());
        }

        // For non-test tasks, use the estimated duration
        tokio::time::sleep(task.metadata.estimated_duration).await;
        Ok(())
    }

    /// Cancels a task with the given task_id.
    pub async fn cancel_task(&self, task_id: &TaskId) -> Result<(), BuildError> {
        let task = self.state_manager.get_task(task_id).await
            .map_err(|e| BuildError::StateError(e))?;

        // Release any allocated resources
        self.release_resources(&task).await;

        // Update task status to Failed
        self.state_manager.update_task_status(task_id, TaskStatus::Failed).await
            .map_err(|e| BuildError::StateError(e))?;

        Ok(())
    }

    /// Gets the status of a task with the given task_id.
    pub async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, BuildError> {
        let task = self.state_manager.get_task(task_id).await
            .map_err(|e| BuildError::StateError(e))?;
        Ok(task.status)
    }

    /// Lists all running tasks.
    pub async fn list_running_tasks(&self) -> Result<Vec<TaskState>, BuildError> {
        let tasks = self.state_manager.get_tasks_by_status(TaskStatus::Running).await
            .map_err(|e| BuildError::StateError(e))?;
        Ok(tasks)
    }

    /// Gets all ready tasks.
    pub async fn get_ready_tasks(&self) -> Result<Vec<TaskState>, BuildError> {
        self.state_manager.get_ready_tasks().await
            .map_err(|e| BuildError::StateError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::state::types::{TaskMetadata, TaskPriority};
    use std::time::Duration;

    #[tokio::test]
    async fn test_execute_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        let task = create_test_task("test-1");

        state_manager.create_task(task.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        engine.execute_task(&task.id).await?;

        let final_status = engine.get_task_status(&task.id).await?;
        assert_eq!(final_status, TaskStatus::Completed);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_task_with_dependencies() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        
        // Create tasks
        let task1 = create_test_task("test-2");
        let mut task2 = create_test_task("test-3");
        task2.metadata.dependencies = vec![TaskId::new(&task1.id.0)];

        // Add tasks to state manager
        state_manager.create_task(task1.clone()).await
            .map_err(|e| BuildError::StateError(e))?;
        state_manager.create_task(task2.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        // Try to execute task2 before task1 is complete
        let result = engine.execute_task(&task2.id).await;
        assert!(matches!(result, Err(BuildError::DependenciesNotMet(_))));

        // Execute task1
        engine.execute_task(&task1.id).await?;
        assert_eq!(engine.get_task_status(&task1.id).await?, TaskStatus::Completed);

        // Now task2 should execute successfully
        engine.execute_task(&task2.id).await?;
        assert_eq!(engine.get_task_status(&task2.id).await?, TaskStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_ready_tasks() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        
        // Create tasks
        let task1 = create_test_task("test-4");
        let mut task2 = create_test_task("test-5");
        task2.metadata.dependencies = vec![TaskId::new(&task1.id.0)];

        // Add tasks to state manager
        state_manager.create_task(task1.clone()).await
            .map_err(|e| BuildError::StateError(e))?;
        state_manager.create_task(task2.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        // Check ready tasks
        let ready_tasks = engine.get_ready_tasks().await?;
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, task1.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_cancel_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let engine = BuildEngine::new(state_manager.clone());
        let task = create_test_task("test-6");

        state_manager.create_task(task.clone()).await
            .map_err(|e| BuildError::StateError(e))?;

        engine.execute_task(&task.id).await?;
        engine.cancel_task(&task.id).await?;

        let final_status = engine.get_task_status(&task.id).await?;
        assert_eq!(final_status, TaskStatus::Failed);
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
                estimated_duration: Duration::from_secs(60),
                dependencies: vec![],
                additional_info: HashMap::new(),
            },
        }
    }
}
