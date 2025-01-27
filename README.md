# Build System

A Rust-based build system with intelligent task management, state tracking, and dependency resolution.

## Features

- Task State Management
  - Async task lifecycle management
  - Dependency resolution with cycle detection
  - Task metadata tracking
  - State persistence and recovery

- Build Engine
  - Async build step execution
  - Task dependency validation
  - Build artifact management

- Documentation Engine
  - Async documentation operations
  - Content validation
  - Version tracking

- Prompt System
  - Template-based prompt generation
  - Async response processing
  - Robust error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
build-system = "0.1.0"
```

## Usage

### Basic Task Management

```rust
use build_system::state::{StateManager, TaskState, TaskStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new state manager
    let state_manager = StateManager::new();

    // Create a task
    let task = TaskState::new("build", "Build project");
    let task_id = state_manager.create_task(task).await?;

    // Update task status
    state_manager.update_task_status(&task_id, TaskStatus::InProgress).await?;

    // Get task by ID
    let task = state_manager.get_task(&task_id).await?;
    println!("Task status: {:?}", task.status);

    Ok(())
}
```

### Managing Task Dependencies

```rust
use build_system::state::{StateManager, TaskState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state_manager = StateManager::new();

    // Create dependent tasks
    let compile_task = TaskState::new("compile", "Compile source");
    let test_task = TaskState::new("test", "Run tests");
    
    let compile_id = state_manager.create_task(compile_task).await?;
    let test_id = state_manager.create_task(test_task).await?;

    // Add dependency
    state_manager.add_dependency(test_id, vec![compile_id]).await?;

    // Get ready tasks (tasks with no pending dependencies)
    let ready_tasks = state_manager.get_ready_tasks().await?;
    
    Ok(())
}
```

### Documentation Management

```rust
use build_system::doc::{Documentation, FileDocumentationEngine};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc_engine = FileDocumentationEngine::new(PathBuf::from("docs"));

    // Create documentation
    let doc = Documentation {
        path: PathBuf::from("build/README.md"),
        content: "# Build Documentation".to_string(),
    };
    
    doc_engine.create_doc(doc).await?;
    
    Ok(())
}
```

## Testing

Run the test suite:

```bash
cargo test
```

## Core Functionality Status

The core functionality is implemented and tested:

✅ Task State Management
- Task creation, updating, and deletion
- Dependency tracking and resolution
- State persistence

✅ Build Engine
- Task execution
- Build step management
- Error handling

✅ Documentation Engine
- Document operations (CRUD)
- Content management

✅ Prompt System
- Template management
- Response processing

The library is ready for testing with basic functionality. However, some planned features are still in development:

🚧 In Progress
- Resource management optimization
- Advanced task scheduling
- Build artifact caching
- Performance monitoring

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
