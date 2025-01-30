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

- AI-Powered Project Generation
  - AI-driven project structure creation
  - Customizable project templates
  - Intelligent dependency selection
  - Cross-language project generation support

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

### AI-Powered Project Generation

#### Inference Client

The build system includes an advanced AI inference client that can:
- Generate project configurations
- Stream AI completions
- Process task prompts with intelligent routing

#### Configuration

Create a `.env` file with the following configuration:

```bash
# OpenAI or compatible API configuration
INFERENCE_API_BASE_URL=https://api.openai.com/v1
INFERENCE_API_KEY=your_api_key
INFERENCE_API_MODEL=gpt-3.5-turbo
INFERENCE_API_TEMPERATURE=0.6
```

#### Example Usage

```rust
use build_system::inference::InferenceClient;
use build_system::prompt::Prompt;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize AI inference client
    let inference_client = InferenceClient::new()?;

    // Create a project generation prompt
    let prompt = Prompt {
        system_context: "You are an expert software architect.".to_string(),
        user_request: "Generate a project structure for a task management app".to_string(),
        build_context: None,
    };

    // Generate project configuration
    let (response, status) = inference_client
        .execute_task_prompt(&prompt, &TaskId::new("project_generation"))
        .await?;
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

✅ AI-Powered Project Generation
- AI-driven project structure creation
- Customizable project templates
- Intelligent dependency selection
- Cross-language project generation support

The library is ready for testing with basic functionality. However, some planned features are still in development:

🚧 In Progress
- Resource management optimization
- Advanced task scheduling
- Build artifact caching
- Performance monitoring

## Troubleshooting

### Common Issues

1. **API Key Not Set**: Ensure `INFERENCE_API_KEY` is correctly configured
2. **Model Compatibility**: Verify the inference API supports your chosen model
3. **Network Issues**: Check internet connectivity and API endpoint

### Logging

Enable detailed logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
