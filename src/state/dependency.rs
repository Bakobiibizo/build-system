use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::error::StateError;
use crate::state::types::TaskId;

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    dependencies: Arc<RwLock<HashMap<TaskId, HashSet<TaskId>>>>,
    dependents: Arc<RwLock<HashMap<TaskId, HashSet<TaskId>>>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            dependents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task_id: TaskId, dependencies: Vec<TaskId>) -> Result<(), StateError> {
        let mut deps = self.dependencies.write().await;
        let mut depts = self.dependents.write().await;

        // Add dependencies for the task
        let task_deps = deps.entry(task_id.clone()).or_insert_with(HashSet::new);
        for dep in &dependencies {
            task_deps.insert(dep.clone());

            // Add task as a dependent for each dependency
            let dep_depts = depts.entry(dep.clone()).or_insert_with(HashSet::new);
            dep_depts.insert(task_id.clone());
        }

        Ok(())
    }

    pub async fn remove_task(&self, task_id: &TaskId) -> Result<(), StateError> {
        let mut deps = self.dependencies.write().await;
        let mut depts = self.dependents.write().await;

        // Remove task from dependencies
        if let Some(task_deps) = deps.remove(task_id) {
            // Remove task from dependents of its dependencies
            for dep in task_deps {
                if let Some(dep_depts) = depts.get_mut(&dep) {
                    dep_depts.remove(task_id);
                }
            }
        }

        // Remove task from dependents
        if let Some(task_depts) = depts.remove(task_id) {
            // Remove task from dependencies of its dependents
            for dept in task_depts {
                if let Some(dept_deps) = deps.get_mut(&dept) {
                    dept_deps.remove(task_id);
                }
            }
        }

        Ok(())
    }

    pub async fn get_dependencies(&self, task_id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        let deps = self.dependencies.read().await;
        Ok(deps.get(task_id).cloned().unwrap_or_default())
    }

    pub async fn get_dependents(&self, task_id: &TaskId) -> Result<HashSet<TaskId>, StateError> {
        let depts = self.dependents.read().await;
        Ok(depts.get(task_id).cloned().unwrap_or_default())
    }

    pub async fn has_cycle(&self) -> bool {
        let deps = self.dependencies.read().await;
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for task_id in deps.keys() {
            if !visited.contains(task_id) {
                if self.check_cycle_dfs(task_id, &deps, &mut visited, &mut stack) {
                    return true;
                }
            }
        }

        false
    }

    fn check_cycle_dfs(
        &self,
        task_id: &TaskId,
        deps: &HashMap<TaskId, HashSet<TaskId>>,
        visited: &mut HashSet<TaskId>,
        stack: &mut HashSet<TaskId>,
    ) -> bool {
        visited.insert(task_id.clone());
        stack.insert(task_id.clone());

        if let Some(task_deps) = deps.get(task_id) {
            for dep in task_deps {
                if !visited.contains(dep) {
                    if self.check_cycle_dfs(dep, deps, visited, stack) {
                        return true;
                    }
                } else if stack.contains(dep) {
                    return true;
                }
            }
        }

        stack.remove(task_id);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_dependencies() {
        let graph = DependencyGraph::new();
        let task1 = TaskId::new("test-1");
        let task2 = TaskId::new("test-2");

        graph
            .add_task(task1.clone(), vec![task2.clone()])
            .await
            .unwrap();

        let deps = graph.get_dependencies(&task1).await.unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&task2));

        let depts = graph.get_dependents(&task2).await.unwrap();
        assert_eq!(depts.len(), 1);
        assert!(depts.contains(&task1));
    }

    #[tokio::test]
    async fn test_remove_task() {
        let graph = DependencyGraph::new();
        let task1 = TaskId::new("test-1");
        let task2 = TaskId::new("test-2");

        graph
            .add_task(task1.clone(), vec![task2.clone()])
            .await
            .unwrap();

        graph.remove_task(&task1).await.unwrap();

        let deps = graph.get_dependencies(&task1).await.unwrap();
        assert_eq!(deps.len(), 0);

        let depts = graph.get_dependents(&task2).await.unwrap();
        assert_eq!(depts.len(), 0);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let graph = DependencyGraph::new();
        let task1 = TaskId::new("test-1");
        let task2 = TaskId::new("test-2");
        let task3 = TaskId::new("test-3");

        // Create a cycle: task1 -> task2 -> task3 -> task1
        graph
            .add_task(task1.clone(), vec![task2.clone()])
            .await
            .unwrap();
        graph
            .add_task(task2.clone(), vec![task3.clone()])
            .await
            .unwrap();
        graph
            .add_task(task3.clone(), vec![task1.clone()])
            .await
            .unwrap();

        assert!(graph.has_cycle().await);
    }
}
