# State Management Specifications

## Implementation Details

### Data Structures

#### Task Management
```rust
pub struct TaskState {
    pub id: TaskId,
    pub status: TaskStatus,
    pub dependencies: Vec<TaskId>,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TaskMetadata {
    pub description: String,
    pub owner: String,
    pub priority: TaskPriority,
    pub tags: Vec<String>,
    pub estimated_duration: Duration,
}
```

### Algorithms

#### Dependency Resolution
1. **Topological Sort**
   ```rust
   fn resolve_dependencies(tasks: &[TaskState]) -> Result<Vec<TaskId>> {
       // Kahn's algorithm for topological sorting
       let mut in_degree = HashMap::new();
       let mut graph = HashMap::new();
       
       // Build graph and calculate in-degrees
       for task in tasks {
           graph.entry(task.id).or_default();
           for dep in &task.dependencies {
               graph.entry(dep).or_default().push(task.id);
               *in_degree.entry(task.id).or_default() += 1;
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
   ```

2. **State Persistence**
   ```rust
   pub struct StateSnapshot {
       pub tasks: HashMap<TaskId, TaskState>,
       pub timestamp: DateTime<Utc>,
       pub version: u32,
   }
   
   impl StateManager {
       pub async fn save_snapshot(&self) -> Result<()> {
           let snapshot = StateSnapshot {
               tasks: self.tasks.clone(),
               timestamp: Utc::now(),
               version: self.version,
           };
           
           let json = serde_json::to_string_pretty(&snapshot)?;
           tokio::fs::write(
               self.snapshot_path.join(format!("state_{}.json", snapshot.version)),
               json
           ).await?;
           
           Ok(())
       }
   }
   ```

### Performance Requirements

#### Latency Targets
- Task creation: < 10ms
- Task status update: < 5ms
- Dependency resolution: < 50ms
- State snapshot: < 100ms

#### Throughput Targets
- Concurrent task updates: 1000/s
- State queries: 5000/s
- Snapshot creation: 1/5min

## Integration Contract

### Public API
```rust
pub trait StateManager {
    async fn create_task(&mut self, task: TaskState) -> Result<TaskId>;
    async fn update_task(&mut self, id: TaskId, status: TaskStatus) -> Result<()>;
    async fn get_task(&self, id: &TaskId) -> Option<&TaskState>;
    async fn list_tasks(&self) -> Vec<&TaskState>;
    async fn resolve_dependencies(&self, task: &TaskId) -> Result<Vec<TaskId>>;
}
```

### Event Protocol
```rust
pub enum StateEvent {
    TaskCreated(TaskId),
    TaskUpdated(TaskId, TaskStatus),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, Error),
    DependencyResolved(TaskId, Vec<TaskId>),
    SnapshotCreated(u32),
}

pub trait StateEventHandler {
    async fn handle_event(&mut self, event: StateEvent) -> Result<()>;
}
```

### Error Contract
```rust
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    #[error("Invalid task state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: TaskStatus, to: TaskStatus },
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Persistence error: {0}")]
    PersistenceError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

## Configuration

### Required Parameters
```toml
[state_manager]
snapshot_dir = "/var/lib/build-system/snapshots"
snapshot_interval = "5m"
max_tasks = 10000
max_dependencies_per_task = 100
persistence_enabled = true

[state_manager.locks]
timeout = "10s"
retry_interval = "100ms"
max_retries = 5

[state_manager.cleanup]
enabled = true
retention_period = "7d"
cleanup_interval = "1h"
```

### Environment Variables
```bash
BUILDSTATE_SNAPSHOT_DIR=/var/lib/build-system/snapshots
BUILDSTATE_MAX_TASKS=10000
BUILDSTATE_PERSISTENCE_ENABLED=true
BUILDSTATE_LOG_LEVEL=info
```

### Resource Requirements
- Memory: 500MB base, 2GB peak
- Disk: 10GB for snapshots
- CPU: 2 cores recommended

## Testing

### Test Data Format
```json
{
    "task": {
        "id": "task-123",
        "status": "pending",
        "dependencies": ["task-121", "task-122"],
        "metadata": {
            "description": "Build frontend assets",
            "owner": "frontend-team",
            "priority": "high",
            "tags": ["frontend", "assets"],
            "estimated_duration": "PT30M"
        }
    }
}
```

### Performance Tests
1. **Concurrent Operations**
   ```rust
   #[tokio::test]
   async fn test_concurrent_operations() {
       let state = Arc::new(RwLock::new(StateManager::new()));
       let mut handles = vec![];
       
       for i in 0..1000 {
           let state = state.clone();
           handles.push(tokio::spawn(async move {
               let task = TaskState::new(format!("task-{}", i));
               state.write().await.create_task(task).await
           }));
       }
       
       let results = futures::future::join_all(handles).await;
       assert!(results.iter().all(|r| r.is_ok()));
   }
   ```

2. **Load Tests**
   ```rust
   #[tokio::test]
   async fn test_load() {
       let state = StateManager::new();
       let start = Instant::now();
       
       for i in 0..10_000 {
           state.create_task(TaskState::new(format!("task-{}", i))).await?;
       }
       
       let duration = start.elapsed();
       assert!(duration < Duration::from_secs(10));
   }
   ```

### Integration Tests
```rust
#[tokio::test]
async fn test_state_persistence() {
    let temp_dir = tempdir()?;
    let state = StateManager::new_with_config(StateConfig {
        snapshot_dir: temp_dir.path().to_path_buf(),
        persistence_enabled: true,
        ..Default::default()
    });
    
    // Create some tasks
    let task1 = state.create_task(TaskState::new("task-1")).await?;
    let task2 = state.create_task(TaskState::new("task-2")).await?;
    
    // Force snapshot
    state.save_snapshot().await?;
    
    // Create new state manager and verify recovery
    let recovered_state = StateManager::new_with_config(StateConfig {
        snapshot_dir: temp_dir.path().to_path_buf(),
        persistence_enabled: true,
        ..Default::default()
    });
    
    assert_eq!(recovered_state.get_task(&task1).await.is_some(), true);
    assert_eq!(recovered_state.get_task(&task2).await.is_some(), true);
}
```
