# Prompt Management Prompts

## Template Creation Prompt
```
You are designing a prompt template for a build system. Given the following requirements:

[TEMPLATE_REQUIREMENTS]

Create a prompt template that:
1. Clearly defines the task context
2. Specifies required input variables
3. Provides response format guidelines
4. Includes error handling instructions

Your response should be in JSON format:
{
    "template_name": "string",
    "template_content": "string",
    "variables": ["var_1", "var_2"],
    "response_format": {
        "schema": {},
        "example": {}
    }
}
```

## Context Optimization Prompt
```
You are optimizing context for an LLM prompt. Given:

1. Available context: [CONTEXT_JSON]
2. Token limit: [TOKEN_LIMIT]
3. Task type: [TASK_TYPE]

Optimize the context by:
1. Ranking context by relevance
2. Selecting most important pieces
3. Maintaining coherent context
4. Staying within token limit

Your response should be in JSON format:
{
    "selected_context": ["context_1"],
    "token_count": 0,
    "relevance_scores": {"context_1": 0.9},
    "omitted_context": ["context_2"]
}
```

## Response Processing Prompt
```
You are processing an LLM response for a build task. Given:

1. Original prompt: [PROMPT_TEXT]
2. LLM response: [RESPONSE_TEXT]
3. Expected format: [FORMAT_SPEC]

Analyze the response and:
1. Validate format compliance
2. Extract key information
3. Identify any errors
4. Suggest corrections if needed

Your response should be in JSON format:
{
    "is_valid": true,
    "extracted_data": {},
    "errors": [],
    "suggested_corrections": []
}
```

## Template Optimization Prompt
```
You are optimizing prompt templates for better LLM responses. Given:

1. Current template: [TEMPLATE_TEXT]
2. Usage statistics: [USAGE_STATS]
3. Success rates: [SUCCESS_RATES]

Suggest improvements for:
1. Clarity and specificity
2. Response consistency
3. Error reduction
4. Token efficiency

Your response should be in JSON format:
{
    "suggested_changes": ["change_1"],
    "expected_improvements": {
        "clarity": 0.8,
        "consistency": 0.9
    },
    "token_impact": 0,
    "validation_rules": ["rule_1"]
}
```

## State Component Construction Prompt
```
# EXECUTE TASK: Implement State Management Component

## Context
- Component: State Management System
- Implementation Status: Initial Phase
- Dependencies: None (Core Component)
- Target Directory: src/state/

## Objectives
1. Create thread-safe state management system
2. Implement persistent storage
3. Add dependency tracking
4. Ensure proper error handling

## Implementation Steps

1. Create Core Data Structures
```rust
// src/state/types.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct TaskId(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: TaskId,
    pub status: TaskStatus,
    pub dependencies: Vec<TaskId>,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub description: String,
    pub owner: String,
    pub priority: TaskPriority,
    pub tags: Vec<String>,
    pub estimated_duration: Duration,
}
```

2. Implement State Manager
```rust
// src/state/manager.rs
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub struct StateManager {
    tasks: RwLock<HashMap<TaskId, TaskState>>,
    version: AtomicU32,
    snapshot_path: PathBuf,
}

impl StateManager {
    pub fn new(snapshot_path: PathBuf) -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            version: AtomicU32::new(0),
            snapshot_path,
        }
    }

    pub async fn create_task(&self, task: TaskState) -> Result<TaskId> {
        let mut tasks = self.tasks.write();
        if tasks.contains_key(&task.id) {
            return Err(anyhow!("Task already exists"));
        }
        tasks.insert(task.id.clone(), task);
        Ok(task.id)
    }

    pub async fn update_task(&self, id: TaskId, status: TaskStatus) -> Result<()> {
        let mut tasks = self.tasks.write();
        let task = tasks.get_mut(&id).context("Task not found")?;
        task.status = status;
        task.updated_at = Utc::now();
        Ok(())
    }

    pub async fn save_snapshot(&self) -> Result<()> {
        let tasks = self.tasks.read();
        let version = self.version.fetch_add(1, Ordering::SeqCst);
        let snapshot = StateSnapshot {
            tasks: tasks.clone(),
            timestamp: Utc::now(),
            version,
        };
        
        let json = serde_json::to_string_pretty(&snapshot)?;
        tokio::fs::write(
            self.snapshot_path.join(format!("state_{}.json", version)),
            json
        ).await?;
        
        Ok(())
    }
}
```

3. Implement Dependency Resolution
```rust
// src/state/dependencies.rs
use std::collections::{HashMap, VecDeque};

impl StateManager {
    pub fn resolve_dependencies(&self, tasks: &[TaskState]) -> Result<Vec<TaskId>> {
        let mut in_degree = HashMap::new();
        let mut graph = HashMap::new();
        
        // Build graph and calculate in-degrees
        for task in tasks {
            graph.entry(task.id.clone()).or_default();
            for dep in &task.dependencies {
                graph.entry(dep.clone())
                    .or_default()
                    .push(task.id.clone());
                *in_degree.entry(task.id.clone()).or_default() += 1;
            }
        }
        
        // Find tasks with no dependencies
        let mut queue: VecDeque<TaskId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();
            
        let mut order = Vec::new();
        
        // Process queue
        while let Some(task) = queue.pop_front() {
            order.push(task.clone());
            
            if let Some(dependents) = graph.get(&task) {
                for dependent in dependents {
                    *in_degree.get_mut(dependent).unwrap() -= 1;
                    if in_degree[dependent] == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        
        // Check for cycles
        if order.len() != tasks.len() {
            return Err(anyhow!("Circular dependency detected"));
        }
        
        Ok(order)
    }
}
```

4. Add Error Types
```rust
// src/state/error.rs
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: TaskStatus, to: TaskStatus },
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Persistence error: {0}")]
    PersistenceError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

5. Create Tests
```rust
// src/state/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_task_creation() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(dir.path().to_path_buf());
        
        let task = TaskState {
            id: TaskId("test1".to_string()),
            status: TaskStatus::Pending,
            dependencies: vec![],
            metadata: TaskMetadata::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let result = manager.create_task(task.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), task.id);
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(dir.path().to_path_buf());
        
        let task1 = TaskState::new("task1");
        let task2 = TaskState::new("task2");
        task2.dependencies.push(task1.id.clone());
        
        let tasks = vec![task1, task2];
        let order = manager.resolve_dependencies(&tasks).unwrap();
        
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], tasks[0].id);
        assert_eq!(order[1], tasks[1].id);
    }
}
```

## Documentation Updates
1. Update ARCHITECTURE.md:
   - Add thread safety details
   - Document persistence strategy
   - Describe dependency resolution

2. Update PROGRESS.md:
   - Mark completed features
   - Update timeline
   - Add new upcoming tasks

## Testing Requirements
1. Unit Tests:
   - Task creation/updates
   - State persistence
   - Dependency resolution
   - Error handling

2. Integration Tests:
   - Full workflow testing
   - Concurrent access
   - Recovery from snapshots

## Performance Validation
1. Measure and validate:
   - Task operation latency (< 10ms)
   - Query response time (< 5ms)
   - Snapshot creation time (< 100ms)

2. Concurrent access testing:
   - Multiple readers
   - Write contention
   - Snapshot impact

## Response Format
```json
{
    "implemented_features": [
        "thread_safe_state",
        "persistence",
        "dependency_tracking"
    ],
    "test_coverage": {
        "unit_tests": [
            "task_creation",
            "task_updates",
            "dependency_resolution"
        ],
        "integration_tests": [
            "concurrent_access",
            "snapshot_recovery"
        ]
    },
    "performance_metrics": {
        "task_ops_latency": "8ms",
        "query_latency": "3ms",
        "snapshot_latency": "45ms"
    },
    "validation_results": [
        "all_tests_passing",
        "performance_targets_met",
        "documentation_updated"
    ]
}
```
