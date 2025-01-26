# Prompt Management Specifications

## Implementation Details

### Data Structures

#### Prompt Management
```rust
pub struct Prompt {
    pub id: PromptId,
    pub template: String,
    pub variables: HashMap<String, String>,
    pub context: PromptContext,
    pub metadata: PromptMetadata,
}

pub struct PromptContext {
    pub system_context: String,
    pub user_context: String,
    pub memory_context: Vec<Memory>,
    pub task_context: Option<TaskContext>,
}

pub struct PromptMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
    pub tags: Vec<String>,
    pub category: PromptCategory,
}

pub struct Memory {
    pub id: String,
    pub content: String,
    pub relevance_score: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PromptCategory {
    TaskCreation,
    BuildExecution,
    Documentation,
    ErrorHandling,
    SystemOptimization,
}
```

### Algorithms

#### Template Processing
```rust
impl PromptManager {
    pub fn process_template(&self, prompt: &Prompt) -> Result<String> {
        let mut handlebars = Handlebars::new();
        
        // Register template helpers
        handlebars.register_helper("format_date", Box::new(format_date_helper));
        handlebars.register_helper("truncate", Box::new(truncate_helper));
        
        // Create template context
        let mut context = HashMap::new();
        context.extend(prompt.variables.clone());
        context.insert("system_context", prompt.context.system_context.clone());
        context.insert("user_context", prompt.context.user_context.clone());
        
        // Process template
        handlebars.render_template(&prompt.template, &context)
            .map_err(|e| anyhow!("Template processing error: {}", e))
    }
    
    fn format_date_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h.param(0).unwrap().value().as_str().unwrap();
        let date = DateTime::parse_from_rfc3339(param).unwrap();
        out.write(&date.format("%Y-%m-%d %H:%M:%S").to_string())?;
        Ok(())
    }
}
```

#### Context Management
```rust
impl PromptManager {
    pub async fn build_context(&self, request: &PromptRequest) -> Result<PromptContext> {
        // Retrieve relevant memories
        let memories = self.memory_store
            .query_memories(&request.query, 5)
            .await?;
            
        // Get task context if available
        let task_context = if let Some(task_id) = &request.task_id {
            Some(self.state_manager.get_task(task_id).await?)
        } else {
            None
        };
        
        // Build context
        Ok(PromptContext {
            system_context: self.load_system_context().await?,
            user_context: request.user_context.clone(),
            memory_context: memories,
            task_context,
        })
    }
    
    pub async fn update_memories(&self, response: &str) -> Result<()> {
        // Extract relevant information from response
        let new_memories = self.extract_memories(response);
        
        // Store new memories
        for memory in new_memories {
            self.memory_store.store_memory(memory).await?;
        }
        
        Ok(())
    }
}
```

### Performance Requirements

#### Latency Targets
- Template processing: < 50ms
- Context building: < 200ms
- Memory retrieval: < 100ms
- Response processing: < 150ms

#### Throughput Targets
- Concurrent prompts: 20+
- Memory queries: 100+ per second
- Template processing: 50+ per second

## Integration Contract

### Public API
```rust
pub trait PromptManager {
    async fn create_prompt(&mut self, request: PromptRequest) -> Result<Prompt>;
    async fn process_prompt(&self, prompt: &Prompt) -> Result<String>;
    async fn process_response(&mut self, response: &str) -> Result<()>;
    async fn get_prompt(&self, id: &PromptId) -> Option<Prompt>;
    async fn list_prompts(&self, category: Option<PromptCategory>) -> Vec<PromptMetadata>;
}

pub trait MemoryStore {
    async fn store_memory(&mut self, memory: Memory) -> Result<()>;
    async fn query_memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    async fn update_memory(&mut self, id: &str, content: &str) -> Result<()>;
    async fn delete_memory(&mut self, id: &str) -> Result<()>;
}
```

### Event Protocol
```rust
pub enum PromptEvent {
    PromptCreated(PromptId),
    PromptProcessed(PromptId, ProcessingStats),
    MemoryStored(String),
    MemoryUpdated(String),
    ContextBuilt(PromptId, ContextStats),
    Error(PromptError),
}

pub struct ProcessingStats {
    pub template_processing_time: Duration,
    pub context_building_time: Duration,
    pub total_time: Duration,
}

pub trait PromptEventHandler {
    async fn handle_event(&mut self, event: PromptEvent) -> Result<()>;
}
```

### Error Contract
```rust
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Template processing error: {0}")]
    TemplateError(String),
    #[error("Context building error: {0}")]
    ContextError(String),
    #[error("Memory store error: {0}")]
    MemoryError(String),
    #[error("Invalid prompt format: {0}")]
    ValidationError(String),
}
```

## Configuration

### Required Parameters
```toml
[prompt_manager]
template_dir = "/var/lib/build-system/templates"
memory_dir = "/var/lib/build-system/memories"
max_context_size = 4096
max_memories_per_query = 5

[prompt_manager.templates]
cache_size = 100
refresh_interval = "1m"
validation_enabled = true

[prompt_manager.memory]
max_memories = 10000
pruning_interval = "1d"
min_relevance_score = 0.5
```

### Environment Variables
```bash
PROMPT_MANAGER_TEMPLATE_DIR=/var/lib/build-system/templates
PROMPT_MANAGER_MEMORY_DIR=/var/lib/build-system/memories
PROMPT_MANAGER_MAX_CONTEXT_SIZE=4096
PROMPT_MANAGER_LOG_LEVEL=info
```

### Resource Requirements
- Memory: 1GB base, 2GB peak
- Disk: 1GB for templates and memories
- CPU: 2 cores recommended

## Testing

### Test Data Format
```json
{
    "prompt_request": {
        "template": "task_creation",
        "variables": {
            "task_name": "build_frontend",
            "description": "Build frontend assets"
        },
        "context": {
            "user_context": "User wants to build frontend",
            "task_id": "task-123"
        }
    }
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_concurrent_prompt_processing() {
    let manager = PromptManager::new(PromptConfig::default());
    let mut handles = vec![];
    
    // Process multiple prompts concurrently
    for i in 0..20 {
        let manager = manager.clone();
        handles.push(tokio::spawn(async move {
            let request = PromptRequest::new(format!("prompt-{}", i));
            manager.create_prompt(request).await
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    assert!(results.iter().all(|r| r.is_ok()));
}

#[tokio::test]
async fn test_memory_performance() {
    let manager = PromptManager::new(PromptConfig::default());
    
    // Create test memories
    for i in 0..1000 {
        let memory = Memory::new(format!("memory-{}", i));
        manager.memory_store.store_memory(memory).await?;
    }
    
    // Measure query performance
    let start = Instant::now();
    let memories = manager.memory_store
        .query_memories("test query", 5)
        .await?;
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_millis(100));
    assert_eq!(memories.len(), 5);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_prompt_lifecycle() {
    let temp_dir = tempdir()?;
    let manager = PromptManager::new(PromptConfig {
        template_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    });
    
    // Create prompt
    let request = PromptRequest::new("test-prompt")
        .with_template("task_creation")
        .with_variables(HashMap::from([
            ("task_name".to_string(), "test_task".to_string()),
        ]));
    
    let prompt = manager.create_prompt(request).await?;
    
    // Process prompt
    let result = manager.process_prompt(&prompt).await?;
    assert!(!result.is_empty());
    
    // Process response
    manager.process_response(&result).await?;
    
    // Verify memories were created
    let memories = manager.memory_store
        .query_memories("test_task", 1)
        .await?;
    assert!(!memories.is_empty());
}
```
