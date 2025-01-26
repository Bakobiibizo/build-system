use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

use crate::state::error::StateError;
use crate::state::types::{TaskId, TaskState, TaskStatus, StateSnapshot};
use crate::state::dependency::DependencyGraph;

#[derive(Debug, Clone)]
pub struct StateManager {
    states: Arc<RwLock<HashMap<TaskId, TaskState>>>,
    dependencies: DependencyGraph,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            dependencies: DependencyGraph::new(),
        }
    }

    pub async fn create_task(&self, task: TaskState) -> Result<(), StateError> {
        // First check dependencies
        for dep_id in &task.metadata.dependencies {
            let states = self.states.read().await;
            if !states.contains_key(dep_id) {
                return Err(StateError::TaskNotFound(dep_id.to_string()));
            }
        }

        // Now add the task
        let mut states = self.states.write().await;
        if states.contains_key(&task.id) {
            return Err(StateError::TaskAlreadyExists(task.id.to_string()));
        }
        states.insert(task.id.clone(), task.clone());
        
        // Add task to dependency graph with its dependencies
        self.dependencies.add_task(task.id, task.metadata.dependencies).await?;
        Ok(())
    }

    pub async fn get_task(&self, id: &TaskId) -> Result<TaskState, StateError> {
        let states = self.states.read().await;
        states
            .get(id)
            .cloned()
            .ok_or_else(|| StateError::TaskNotFound(id.to_string()))
    }

    pub async fn update_task_status(&self, id: &TaskId, status: TaskStatus) -> Result<(), StateError> {
        // First verify task exists
        let task = {
            let states = self.states.read().await;
            states.get(id).cloned().ok_or_else(|| StateError::TaskNotFound(id.to_string()))?
        };

        // For Completed status, verify all dependencies are completed
        if status == TaskStatus::Completed {
            let deps = self.get_task_dependencies(id).await?;
            for dep_id in deps {
                let dep_status = {
                    let states = self.states.read().await;
                    states.get(&dep_id)
                        .ok_or_else(|| StateError::TaskNotFound(dep_id.to_string()))?
                        .status
                };
                if dep_status != TaskStatus::Completed {
                    return Err(StateError::DependenciesNotMet(id.to_string()));
                }
            }
        }

        // Now update the status
        let mut states = self.states.write().await;
        if let Some(task) = states.get_mut(id) {
            task.status = status;
            task.updated_at = Utc::now();
            Ok(())
        } else {
            Err(StateError::TaskNotFound(id.to_string()))
        }
    }

    pub async fn delete_task(&self, id: &TaskId) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        if states.remove(id).is_some() {
            Ok(())
        } else {
            Err(StateError::TaskNotFound(id.to_string()))
        }
    }

    pub async fn list_tasks(&self) -> Result<Vec<TaskState>, StateError> {
        let states = self.states.read().await;
        Ok(states.values().cloned().collect())
    }

    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<TaskState>, StateError> {
        let states = self.states.read().await;
        Ok(states
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    pub async fn get_ready_tasks(&self) -> Result<Vec<TaskState>, StateError> {
        let ready_ids = self.dependencies.get_ready_tasks().await;
        let mut ready_tasks = Vec::new();
        let states = self.states.read().await;
        
        for id in ready_ids {
            if let Some(task) = states.get(&id) {
                ready_tasks.push(task.clone());
            }
        }
        
        Ok(ready_tasks)
    }

    pub async fn get_task_dependencies(&self, id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        self.dependencies.get_dependencies(id).await
    }

    pub async fn get_task_dependents(&self, id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        self.dependencies.get_dependents(id).await
    }

    pub async fn take_snapshot(&self) -> Result<StateSnapshot, StateError> {
        let states = self.states.read().await;
        Ok(StateSnapshot {
            version: 1,
            timestamp: Utc::now(),
            tasks: states.clone(),
        })
    }

    pub async fn restore_snapshot(&self, snapshot: StateSnapshot) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        *states = snapshot.tasks;
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{TaskMetadata, TaskPriority};
    use std::time::Duration;

    #[tokio::test]
    async fn test_create_and_get_task() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = create_test_task("test-1");

        manager.create_task(task.clone()).await?;
        let retrieved = manager.get_task(&task.id).await?;
        assert_eq!(retrieved, task);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_task_status() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = create_test_task("test-2");
        manager.create_task(task.clone()).await?;

        manager.update_task_status(&task.id, TaskStatus::Running).await?;
        let updated = manager.get_task(&task.id).await?;
        assert_eq!(updated.status, TaskStatus::Running);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_task() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = create_test_task("test-3");
        manager.create_task(task.clone()).await?;

        manager.delete_task(&task.id).await?;
        let result = manager.get_task(&task.id).await;
        assert!(matches!(result, Err(StateError::TaskNotFound(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task1 = create_test_task("test-4");
        let task2 = create_test_task("test-5");

        manager.create_task(task1).await?;
        manager.create_task(task2).await?;

        let tasks = manager.list_tasks().await?;
        assert_eq!(tasks.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_task_dependencies() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task1 = create_test_task("test-6");
        let task2 = create_test_task("test-7");

        // Create task1 with no dependencies
        manager.create_task(task1.clone()).await?;

        // Create task2 with task1 as dependency
        let mut task2 = task2;
        task2.metadata.dependencies = vec![TaskId::new(&task1.id.0)];
        manager.create_task(task2.clone()).await?;

        // Check dependencies
        let deps = manager.get_task_dependencies(&task2.id).await?;
        assert!(deps.contains(&task1.id));

        // Check dependents
        let dependents = manager.get_task_dependents(&task1.id).await?;
        assert!(dependents.contains(&task2.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_ready_tasks() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task1 = create_test_task("test-8");
        let task2 = create_test_task("test-9");

        // Create task1 with no dependencies
        manager.create_task(task1.clone()).await?;

        // Create task2 with task1 as dependency
        let mut task2 = task2;
        task2.metadata.dependencies = vec![TaskId::new(&task1.id.0)];
        manager.create_task(task2.clone()).await?;

        // Check ready tasks
        let ready_tasks = manager.get_ready_tasks().await?;
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, task1.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_snapshot_restore() -> Result<(), StateError> {
        let manager = StateManager::new();
        let task = create_test_task("test-10");
        manager.create_task(task.clone()).await?;

        let snapshot = manager.take_snapshot().await?;
        manager.delete_task(&task.id).await?;
        manager.restore_snapshot(snapshot).await?;

        let restored = manager.get_task(&task.id).await?;
        assert_eq!(restored, task);
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
