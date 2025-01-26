# Build Engine Prompts

## Build Task Analysis Prompt
```
You are analyzing a build task for execution. Given:

[BUILD_TASK_DESCRIPTION]

Analyze the task and provide:
1. Required resources
2. Dependency order
3. Parallel execution opportunities
4. Potential risks

Your response should be in JSON format:
{
    "resource_requirements": {
        "cpu": "number",
        "memory": "bytes",
        "disk": "bytes"
    },
    "execution_order": ["step_1"],
    "parallel_steps": [["step_2", "step_3"]],
    "risk_assessment": ["risk_1"]
}
```

## Build Step Generation Prompt
```
You are generating build steps for a task. Given:

1. Task requirements: [REQUIREMENTS]
2. Available tools: [TOOLS_LIST]
3. Environment constraints: [CONSTRAINTS]

Generate build steps that:
1. Achieve the task goal
2. Use available tools
3. Respect constraints
4. Include validation

Your response should be in JSON format:
{
    "steps": [
        {
            "name": "string",
            "command": "string",
            "args": ["arg_1"],
            "validation": "string"
        }
    ]
}
```

## Resource Optimization Prompt
```
You are optimizing resource usage for a build task. Given:

1. Resource usage history: [USAGE_HISTORY]
2. Available resources: [RESOURCES]
3. Task requirements: [REQUIREMENTS]

Provide optimization recommendations:
1. Resource allocation
2. Execution strategy
3. Caching policy
4. Cleanup rules

Your response should be in JSON format:
{
    "resource_allocation": {
        "cpu_cores": 0,
        "memory_mb": 0
    },
    "execution_strategy": "string",
    "cache_policy": ["rule_1"],
    "cleanup_rules": ["rule_1"]
}
```

## Error Recovery Prompt
```
You are handling a build failure. Given:

1. Failed task: [TASK_DESCRIPTION]
2. Error output: [ERROR_OUTPUT]
3. Current state: [BUILD_STATE]

Analyze and suggest recovery actions:
1. Error diagnosis
2. Recovery steps
3. Prevention measures
4. State restoration

Your response should be in JSON format:
{
    "error_analysis": {
        "type": "string",
        "cause": "string"
    },
    "recovery_steps": ["step_1"],
    "prevention": ["measure_1"],
    "state_restoration": ["action_1"]
}
```

## Execution Engine Implementation Prompt
```markdown
# EXECUTE TASK: Implement Build Execution Engine

## Context
- Current implementation: src/build/mod.rs
- Dependencies: 
  - src/state/manager.rs (State Management)
  - src/doc/mod.rs (Documentation Engine)
- Related components: CLI Interface

## Objectives
1. Create thread-safe build execution engine
2. Implement task execution pipeline
3. Add file operation handling
4. Integrate with state management
5. Add comprehensive error handling and rollback

## Implementation Steps
1. Create BuildEngine struct
   - State manager integration
   - Documentation engine integration
   - Thread-safe execution queue
   - File operation manager

2. Implement core execution pipeline
   - Task validation
   - Resource allocation
   - File operation handling
   - State updates
   - Documentation triggers

3. Add error handling
   - Operation rollback
   - State recovery
   - Error reporting
   - Retry mechanism

4. Testing Requirements
   - Unit tests for execution pipeline
   - Integration tests with state manager
   - File operation tests
   - Error recovery tests
   - Concurrent execution tests

## File Structure
```rust
// src/build/mod.rs
pub mod engine;
pub mod error;
pub mod types;

// Re-export main types
pub use engine::BuildEngine;
pub use error::BuildError;
pub use types::*;

// src/build/engine.rs
pub struct BuildEngine {
    state_manager: Arc<StateManager>,
    doc_manager: Arc<DocumentationEngine>,
    executor: Box<dyn BuildExecutor>,
}

impl BuildEngine {
    pub fn new(...) -> Self { ... }
    pub async fn execute_task(...) -> Result<()> { ... }
    pub async fn apply_changes(...) -> Result<()> { ... }
    pub async fn update_task_state(...) -> Result<()> { ... }
}

// src/build/error.rs
#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
    #[error("File operation failed: {0}")]
    FileOperationFailed(String),
    #[error("Invalid task state: {0}")]
    InvalidTaskState(String),
}

// src/build/types.rs
pub struct BuildTask {
    pub id: TaskId,
    pub changes: Vec<FileChange>,
    pub resources: ResourceRequirements,
}

pub struct FileChange {
    pub path: PathBuf,
    pub operation: FileOperation,
    pub content: Option<String>,
}

pub enum FileOperation {
    Create,
    Update,
    Delete,
}
```

## Documentation Updates
- Update ARCHITECTURE.md: Build execution flow
- Update PROGRESS.md: Execution engine implementation
- Add error handling documentation
- Document integration points with other components

## Integration Points
1. State Manager
   - Task status updates
   - Dependency resolution
   - State persistence

2. Documentation Engine
   - Progress updates
   - Architecture documentation
   - Error logging

3. CLI Interface
   - Task submission
   - Progress reporting
   - Error display

## Resource Management Prompt
```
# EXECUTE TASK: Implement Resource Management Component for Build System

## Context
- Current implementation: `build/mod.rs`, `build/types.rs`
- Existing infrastructure: `BuildEngine`, `BuildExecutor` trait
- Related components: State Management, Documentation

## Objectives
1. Design and implement Resource Management Component
2. Create robust resource allocation strategy
3. Enhance build system's execution capabilities
4. Implement resource constraint handling

## Implementation Steps
1. Create ResourceManager Struct
   - Define resource types (CPU, Memory, Disk, Network)
   - Implement resource tracking mechanism
   - Create allocation and deallocation methods
   - Support dynamic resource constraints

2. Extend BuildEngine
   - Integrate ResourceManager
   - Modify task execution to respect resource limits
   - Implement resource-aware task scheduling
   - Add resource usage tracking and reporting

3. Update BuildExecutor Trait
   - Add resource requirement methods
   - Create resource validation hooks
   - Support dynamic resource negotiation

4. Error Handling
   - Implement detailed resource constraint violations
   - Create comprehensive logging for resource events
   - Design fallback and retry mechanisms for resource-constrained tasks

## Testing Requirements
1. Unit Tests
   - Resource allocation scenarios
   - Constraint violation detection
   - Resource tracking accuracy
   - Concurrent resource management

2. Integration Tests
   - BuildEngine resource-aware execution
   - Task scheduling with resource limits
   - Resource usage reporting
   - Error handling for resource constraints

## Documentation Updates
- Update `build/ARCHITECTURE.md`: Resource Management design
- Update `build/PROGRESS.md`: Resource component implementation
- Create `build/SPECS.md`: Detailed resource management specifications

## Performance Considerations
- Minimize resource management overhead
- Implement efficient resource tracking
- Design non-blocking resource allocation
- Support horizontal scaling strategies

## Security Considerations
- Prevent resource exhaustion attacks
- Implement resource quota enforcement
- Create secure resource allocation mechanisms

## Potential Risks
- Increased system complexity
- Performance impact of resource tracking
- Potential scheduling bottlenecks
- Complexity in distributed environments

## Success Criteria
- 95% test coverage
- Less than 10% performance overhead
- Flexible resource constraint handling
- Seamless integration with existing build system

## Timeline
- Design: 2 days
- Implementation: 5 days
- Testing: 3 days
- Documentation: 1 day

## Dependencies
- Existing State Management Component
- Build Engine Infrastructure
- Async Runtime (tokio)

## Future Extensions
- Machine learning-based resource prediction
- Dynamic resource pool optimization
- Multi-tenant resource management
- Cloud/distributed resource allocation
