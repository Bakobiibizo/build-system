use std::path::PathBuf;
use tokio::process::Command;

use crate::state::types::{TaskId, TaskState, TaskStatus};
use crate::state::StateManager;

pub mod error;
pub use error::BuildError;

#[derive(Debug, Clone)]
pub struct BuildManager {
    pub state_manager: StateManager,
    working_dir: PathBuf,
}

impl BuildManager {
    pub fn new(state_manager: StateManager, working_dir: PathBuf) -> Self {
        Self {
            state_manager,
            working_dir,
        }
    }

    pub async fn execute_task(&self, task_id: &TaskId) -> Result<(), BuildError> {
        // Get task from state manager
        let task = self.state_manager.get_task(task_id).await
            .map_err(BuildError::StateError)?;

        // Execute task command
        self.execute_command(&task).await?;

        // Update task status to completed
        self.state_manager.update_task_status(task_id, TaskStatus::Completed).await
            .map_err(BuildError::StateError)?;

        Ok(())
    }

    async fn execute_command(&self, task: &TaskState) -> Result<(), BuildError> {
        let command = &task.metadata.name;
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            return Err(BuildError::InvalidCommand("Empty command".to_string()));
        }

        let output = Command::new(args[0])
            .args(&args[1..])
            .current_dir(&self.working_dir)
            .output()
            .await?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(BuildError::CommandFailed(error_message));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::Utc;

    #[tokio::test]
    async fn test_execute_task() -> Result<(), BuildError> {
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager.clone(), PathBuf::from("/tmp"));

        // Create a test task
        let task_id = TaskId::new("test-task");
        let task = TaskState {
            id: task_id.clone(),
            status: TaskStatus::Pending,
            metadata: crate::state::types::TaskMetadata {
                name: "echo test".to_string(),
                description: Some("A test task".to_string()),
                owner: "test".to_string(),
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(60),
                priority: 1,
                tags: vec!["test".to_string()],
                additional_info: std::collections::HashMap::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        state_manager.create_task(task).await.map_err(BuildError::StateError)?;
        build_manager.execute_task(&task_id).await?;

        Ok(())
    }
}
