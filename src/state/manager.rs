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
        StateManager {
            states: Arc::new(RwLock::new(HashMap::new())),
            dependencies: DependencyGraph::new(),
        }
    }

    pub async fn create_task(&self, task: TaskState) -> Result<(), StateError> {
        let task_id = task.id.clone();
        let mut states = self.states.write().await;
        if states.contains_key(&task_id) {
            return Err(StateError::TaskAlreadyExists(task_id.to_string()));
        }
        states.insert(task_id, task);
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
        let states = self.states.read().await;
        Ok(states
            .values()
            .filter(|task| {
                task.status == TaskStatus::Pending && task.metadata.dependencies.is_empty()
            })
            .cloned()
            .collect())
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
            tasks: states.clone(),
            timestamp: Utc::now(),
        })
    }

    pub async fn restore_snapshot(&self, snapshot: StateSnapshot) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        *states = snapshot.tasks;
        Ok(())
    }

    pub async fn add_dependency(&self, task_id: TaskId, dependencies: Vec<TaskId>) -> Result<(), StateError> {
        self.dependencies.add_task(task_id, dependencies).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::TaskMetadata;
    use std::time::Duration;

    fn create_test_task(id: &str) -> TaskState {
        let task_id = TaskId::new(id);
        let mut task = TaskState::new(task_id);
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
        task
    }

    #[tokio::test]
    async fn test_create_and_get_task() {
        let manager = StateManager::new();
        let task = create_test_task("test-task-1");
        
        // Create task
        manager.create_task(task.clone()).await.unwrap();
        
        // Get task
        let retrieved = manager.get_task(&task.id).await.unwrap();
        assert_eq!(retrieved, task);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let manager = StateManager::new();
        let task = create_test_task("test-task-1");
        
        // Create task
        manager.create_task(task.clone()).await.unwrap();
        
        // Update status
        manager.update_task_status(&task.id, TaskStatus::Running).await.unwrap();
        
        // Verify status
        let updated = manager.get_task(&task.id).await.unwrap();
        assert_eq!(updated.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn test_get_tasks_by_status() {
        let manager = StateManager::new();
        
        // Create tasks with different statuses
        let task1 = create_test_task("test-task-1");
        let task2 = create_test_task("test-task-2");
        
        manager.create_task(task1).await.unwrap();
        manager.create_task(task2).await.unwrap();
        
        manager.update_task_status(&TaskId::new("test-task-1"), TaskStatus::Running).await.unwrap();
        
        // Get running tasks
        let running = manager.get_tasks_by_status(TaskStatus::Running).await.unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id.0, "test-task-1");
    }

    #[tokio::test]
    async fn test_get_ready_tasks() {
        let manager = StateManager::new();
        
        // Create tasks
        let task1 = create_test_task("test-task-1");
        let mut task2 = create_test_task("test-task-2");
        task2.metadata.dependencies.push(TaskId::new("test-task-1"));
        
        manager.create_task(task1).await.unwrap();
        manager.create_task(task2).await.unwrap();
        
        // Get ready tasks
        let ready = manager.get_ready_tasks().await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id.0, "test-task-1");
    }
}
