# State Management Prompts

## Task Creation Prompt
```
You are managing the state of a build system. Given the following build task description:

[TASK_DESCRIPTION]

Create a new TaskState with the following requirements:
1. Generate a unique TaskId
2. Set initial status to Pending
3. Identify and list all task dependencies
4. Ensure no circular dependencies are created
5. Validate all referenced dependencies exist

Your response should be in JSON format:
{
    "task_id": "string",
    "status": "Pending",
    "dependencies": ["dep_id_1", "dep_id_2"],
    "validation_errors": []
}
```

## Dependency Resolution Prompt
```
You are analyzing build task dependencies. Given the following task graph:

[TASK_GRAPH]

1. Identify the optimal build order
2. Detect any circular dependencies
3. Group independent tasks that can be run in parallel
4. Estimate the critical path

Your response should be in JSON format:
{
    "build_order": ["task_1", "task_2"],
    "parallel_groups": [["task_3", "task_4"]],
    "circular_deps": [],
    "critical_path": ["task_1", "task_2"]
}
```

## State Recovery Prompt
```
You are recovering from a build system state corruption. Given the following:

1. Last known good state: [STATE_JSON]
2. Corrupted state: [CORRUPTED_STATE_JSON]
3. Build logs: [LOG_CONTENT]

Analyze and suggest state recovery actions:
1. Identify recoverable tasks
2. Detect irrecoverable state
3. Suggest cleanup actions
4. Provide recovery steps

Your response should be in JSON format:
{
    "recoverable_tasks": ["task_1"],
    "irrecoverable_tasks": ["task_2"],
    "cleanup_actions": ["action_1"],
    "recovery_steps": ["step_1"]
}
```

## State Optimization Prompt
```
You are optimizing the build system state. Given the following metrics:

1. Current memory usage: [MEMORY_USAGE]
2. Access patterns: [ACCESS_PATTERNS]
3. Task completion history: [TASK_HISTORY]

Suggest optimization strategies:
1. Identify candidates for cleanup
2. Recommend caching strategies
3. Suggest state compaction methods
4. Propose index optimizations

Your response should be in JSON format:
{
    "cleanup_candidates": ["task_1"],
    "caching_strategy": "strategy_description",
    "compaction_methods": ["method_1"],
    "index_optimizations": ["optimization_1"]
}
```

## State Implementation Prompt
```
You are implementing the core state management system. Given the following requirements:

1. Current Implementation Status:
   - Basic state management structs defined
   - Task ID and state structures
   - Initial HashMap-based storage
   - Basic task status tracking

2. Implementation Requirements:
   - Add thread-safe state access using RwLock
   - Implement persistent state storage with JSON serialization
   - Add state recovery mechanisms
   - Implement basic task dependency tracking

Implement the following components in order:

1. Thread-Safe State Container:
   ```rust
   pub struct StateManager {
       tasks: RwLock<HashMap<TaskId, TaskState>>,
       version: AtomicU32,
       snapshot_path: PathBuf,
   }
   ```

2. Persistence Layer:
   - Implement JSON serialization for TaskState
   - Add snapshot creation mechanism
   - Create state recovery functionality
   - Handle serialization errors

3. Basic Dependency Tracking:
   - Implement dependency validation
   - Add circular dependency detection
   - Create basic build order resolution

Performance Requirements:
- Task operations: < 10ms
- State queries: < 5ms
- Snapshot creation: < 100ms

Error Handling:
- Implement proper error propagation
- Add state validation
- Handle concurrent access errors
- Manage serialization failures

Your implementation should follow these guidelines:
1. Use proper error types and Result
2. Implement proper logging
3. Add comprehensive testing
4. Document all public APIs
5. Follow Rust best practices

Response Format:
```json
{
    "implemented_features": ["feature1", "feature2"],
    "test_coverage": {
        "unit_tests": ["test1", "test2"],
        "integration_tests": ["test3"]
    },
    "performance_metrics": {
        "task_ops_latency": "Xms",
        "query_latency": "Yms",
        "snapshot_latency": "Zms"
    },
    "validation_results": ["check1", "check2"]
}
```
