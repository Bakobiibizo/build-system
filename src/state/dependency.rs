use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::error::StateError;
use crate::state::types::TaskId;

/// DependencyGraph manages task dependencies and provides methods for dependency resolution
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    nodes: Arc<RwLock<HashMap<TaskId, HashSet<TaskId>>>>,  // task -> dependencies
    reverse: Arc<RwLock<HashMap<TaskId, HashSet<TaskId>>>>, // task -> dependents
}

impl DependencyGraph {
    /// Creates a new empty DependencyGraph
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            reverse: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a task and its dependencies to the graph
    /// Returns error if adding would create a circular dependency
    pub async fn add_task(&self, task_id: TaskId, dependencies: Vec<TaskId>) -> Result<(), StateError> {
        // First check for circular dependencies to avoid deadlock
        let deps_set: HashSet<TaskId> = dependencies.iter().cloned().collect();
        if !deps_set.is_empty() && self.would_create_cycle(&task_id, &deps_set).await {
            return Err(StateError::CircularDependency(task_id.to_string()));
        }

        // Now acquire write locks
        let mut nodes = self.nodes.write().await;
        let mut reverse = self.reverse.write().await;

        // Add task and its dependencies
        nodes.insert(task_id.clone(), deps_set.clone());

        // Update reverse dependencies
        for dep in deps_set {
            reverse
                .entry(dep)
                .or_insert_with(HashSet::new)
                .insert(task_id.clone());
        }

        Ok(())
    }

    /// Removes a task and its dependencies from the graph
    pub async fn remove_task(&self, task_id: &TaskId) -> Result<(), StateError> {
        let mut nodes = self.nodes.write().await;
        let mut reverse = self.reverse.write().await;

        // Remove from nodes
        if let Some(deps) = nodes.remove(task_id) {
            // Remove task from reverse dependencies
            for dep in deps {
                if let Some(dependents) = reverse.get_mut(&dep) {
                    dependents.remove(task_id);
                }
            }
        }

        // Remove from reverse map
        reverse.remove(task_id);

        Ok(())
    }

    /// Gets the dependencies of a task
    pub async fn get_dependencies(&self, task_id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        let nodes = self.nodes.read().await;
        nodes
            .get(task_id)
            .cloned()
            .ok_or_else(|| StateError::TaskNotFound(task_id.to_string()))
    }

    /// Gets the tasks that depend on the given task
    pub async fn get_dependents(&self, task_id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        let reverse = self.reverse.read().await;
        Ok(reverse.get(task_id).cloned().unwrap_or_default())
    }

    /// Gets tasks that have no dependencies and are ready to execute
    pub async fn get_ready_tasks(&self) -> Vec<TaskId> {
        let nodes = self.nodes.read().await;
        let reverse = self.reverse.read().await;
        
        nodes.iter()
            .filter(|(task_id, deps)| {
                deps.is_empty() || deps.iter().all(|dep_id| {
                    !reverse.contains_key(dep_id) || 
                    reverse.get(dep_id).map(|deps| deps.is_empty()).unwrap_or(true)
                })
            })
            .map(|(task_id, _)| task_id.clone())
            .collect()
    }

    /// Checks if adding new dependencies would create a cycle
    async fn would_create_cycle(&self, task_id: &TaskId, new_deps: &HashSet<TaskId>) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with the new dependencies
        queue.extend(new_deps.iter().cloned());

        while let Some(current) = queue.pop_front() {
            if current == *task_id {
                return true; // Found a cycle
            }

            if visited.insert(current.clone()) {
                // Release lock after getting dependencies
                let deps = {
                    let nodes = self.nodes.read().await;
                    nodes.get(&current).cloned().unwrap_or_default()
                };
                queue.extend(deps);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{TaskMetadata, TaskPriority, TaskState, TaskStatus};
    use std::time::Duration;
    use chrono::Utc;

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

    #[tokio::test]
    async fn test_add_and_get_dependencies() -> Result<(), StateError> {
        let graph = DependencyGraph::new();
        let task1 = create_test_task("test-1");
        let task2 = create_test_task("test-2");
        
        let mut deps = Vec::new();
        deps.push(task2.id.clone());
        
        graph.add_task(task1.id.clone(), deps).await?;
        
        let result = graph.get_dependencies(&task1.id).await?;
        assert!(result.contains(&task2.id));
        Ok(())
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() -> Result<(), StateError> {
        let graph = DependencyGraph::new();
        let task1 = create_test_task("test-3");
        let task2 = create_test_task("test-4");
        
        let mut deps1 = Vec::new();
        deps1.push(task2.id.clone());
        graph.add_task(task1.id.clone(), deps1).await?;
        
        let mut deps2 = Vec::new();
        deps2.push(task1.id.clone());
        let result = graph.add_task(task2.id.clone(), deps2).await;
        
        assert!(matches!(result, Err(StateError::CircularDependency(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_ready_tasks() -> Result<(), StateError> {
        let graph = DependencyGraph::new();
        let task1 = create_test_task("test-5");
        let task2 = create_test_task("test-6");
        
        graph.add_task(task1.id.clone(), Vec::new()).await?;
        
        let mut deps = Vec::new();
        deps.push(task1.id.clone());
        graph.add_task(task2.id.clone(), deps).await?;
        
        let ready_tasks = graph.get_ready_tasks().await;
        assert_eq!(ready_tasks.len(), 1);
        assert!(ready_tasks.contains(&task1.id));
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_task() -> Result<(), StateError> {
        let graph = DependencyGraph::new();
        let task1 = create_test_task("test-7");
        let task2 = create_test_task("test-8");
        
        let mut deps = Vec::new();
        deps.push(task2.id.clone());
        
        graph.add_task(task1.id.clone(), deps).await?;
        graph.remove_task(&task1.id).await?;
        
        let result = graph.get_dependencies(&task1.id).await;
        assert!(matches!(result, Err(StateError::TaskNotFound(_))));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_dependents() -> Result<(), StateError> {
        let graph = DependencyGraph::new();
        let task1 = create_test_task("test-9");
        let task2 = create_test_task("test-10");
        
        let mut deps = Vec::new();
        deps.push(task1.id.clone());
        graph.add_task(task2.id.clone(), deps).await?;
        
        let dependents = graph.get_dependents(&task1.id).await?;
        assert_eq!(dependents.len(), 1);
        assert!(dependents.contains(&task2.id));
        Ok(())
    }
}
