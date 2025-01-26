use std::sync::Arc;
use std::path::PathBuf;

use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use chrono::Utc;

use build_system::{
    build::{
        BuildEngine,
        error::BuildError,
        types::{
            BuildTask,
            BuildExecutor,
            ResourceRequirements,
            ResourceConstraint,
            FileChange,
            ResourceAllocation,
        },
    },
    state::{
        TaskId,
        TaskStatus,
        TaskMetadata,
        StateManager,
        error::StateError,
    },
    doc::{
        DocumentationEngine,
        DocumentationError,
        types::{
            Documentation,
            DocType,
            DocumentationStep,
            DocumentationStepStatus,
        },
    },
};

mock! {
    BuildExecutor {}
    
    #[async_trait]
    impl BuildExecutor for BuildExecutor {
        async fn execute_task(&self, task: BuildTask) -> Result<(), BuildError>;
        async fn get_task_status(&self, id: &str) -> Result<TaskStatus, BuildError>;
        async fn cancel_task(&self, id: &str) -> Result<(), BuildError>;
        async fn apply_changes(&self, changes: &[FileChange]) -> Result<(), BuildError>;
        async fn check_resource_availability(&self, requirements: &ResourceRequirements) -> Result<bool, BuildError>;
    }
}

mock! {
    StateManager {}
    
    #[async_trait]
    impl StateManager for StateManager {
        async fn update_task_status(&self, id: TaskId, status: TaskStatus) -> Result<(), StateError>;
        async fn create_task(&self, task: TaskState) -> Result<TaskId, StateError>;
        async fn get_task(&self, id: &TaskId) -> Result<TaskState, StateError>;
        async fn resolve_dependencies(&self, tasks: &[TaskState]) -> Result<Vec<TaskId>, StateError>;
    }
}

mock! {
    DocumentationEngine {}
    
    #[async_trait]
    impl DocumentationEngine for DocumentationEngine {
        async fn update_doc(&self, doc: Documentation) -> Result<(), DocumentationError>;
        async fn read_doc(&self, path: PathBuf, doc_type: DocType) -> Result<Documentation, DocumentationError>;
    }
}

#[tokio::test]
async fn test_build_task_execution() -> Result<(), BuildError> {
    let mut mock_executor = MockBuildExecutor::new();
    let mut mock_state_manager = MockStateManager::new();
    let mut mock_doc_engine = MockDocumentationEngine::new();

    let task = BuildTask {
        id: "test-task-1".to_string(),
        resources: ResourceRequirements {
            cpu: ResourceConstraint { min: 2.0, max: 4.0 },
            memory: ResourceConstraint { min: 4096.0, max: 8192.0 },
            disk: ResourceConstraint { min: 10240.0, max: 20480.0 },
        },
        changes: vec![],
        metadata: TaskMetadata {
            name: "Test Task".to_string(),
            description: Some("Test task description".to_string()),
            owner: "test-user".to_string(),
            priority: TaskPriority::Medium,
            tags: vec!["test".to_string()],
            estimated_duration: Duration::from_secs(3600),
            dependencies: vec![],
            additional_info: HashMap::new(),
        },
    };

    let task_clone = task.clone();
    mock_executor
        .expect_execute_task()
        .with(eq(task_clone))
        .returning(|_| Ok(()));

    mock_executor
        .expect_get_task_status()
        .returning(|_| Ok(TaskStatus::Running));

    let build_engine = BuildEngine::new(
        Arc::new(mock_state_manager),
        Arc::new(mock_doc_engine),
        Arc::new(mock_executor),
        ResourceAllocation {
            cpu_cores: 4,
            memory_mb: 8192,
            disk_gb: 100,
        },
    );

    build_engine.execute_task(task).await?;
    Ok(())
}

#[tokio::test]
async fn test_build_task_failure() -> Result<(), BuildError> {
    let mut mock_executor = MockBuildExecutor::new();
    let mut mock_state_manager = MockStateManager::new();
    let mut mock_doc_engine = MockDocumentationEngine::new();

    let task = BuildTask {
        id: "test-task-1".to_string(),
        resources: ResourceRequirements {
            cpu: ResourceConstraint { min: 2.0, max: 4.0 },
            memory: ResourceConstraint { min: 4096.0, max: 8192.0 },
            disk: ResourceConstraint { min: 10240.0, max: 20480.0 },
        },
        changes: vec![],
        metadata: TaskMetadata {
            name: "Test Task".to_string(),
            description: Some("Test task description".to_string()),
            owner: "test-user".to_string(),
            priority: TaskPriority::Medium,
            tags: vec!["test".to_string()],
            estimated_duration: Duration::from_secs(3600),
            dependencies: vec![],
            additional_info: HashMap::new(),
        },
    };

    let task_clone = task.clone();
    mock_executor
        .expect_execute_task()
        .with(eq(task_clone))
        .returning(|_| Err(BuildError::ExecutionFailed("Task execution failed".to_string())));

    let build_engine = BuildEngine::new(
        Arc::new(mock_state_manager),
        Arc::new(mock_doc_engine),
        Arc::new(mock_executor),
        ResourceAllocation {
            cpu_cores: 4,
            memory_mb: 8192,
            disk_gb: 100,
        },
    );

    let result = build_engine.execute_task(task).await;
    assert!(result.is_err());
    match result {
        Err(BuildError::ExecutionFailed(_)) => Ok(()),
        _ => panic!("Expected ExecutionFailed error"),
    }
}

#[tokio::test]
async fn test_resource_availability() -> Result<(), BuildError> {
    let mut mock_executor = MockBuildExecutor::new();
    let mut mock_state_manager = MockStateManager::new();
    let mut mock_doc_engine = MockDocumentationEngine::new();

    let task = BuildTask {
        id: "test-task-1".to_string(),
        resources: ResourceRequirements {
            cpu: ResourceConstraint { min: 2.0, max: 4.0 },
            memory: ResourceConstraint { min: 4096.0, max: 8192.0 },
            disk: ResourceConstraint { min: 10240.0, max: 20480.0 },
        },
        changes: vec![],
        metadata: TaskMetadata {
            name: "Test Task".to_string(),
            description: Some("Test task description".to_string()),
            owner: "test-user".to_string(),
            priority: TaskPriority::Medium,
            tags: vec!["test".to_string()],
            estimated_duration: Duration::from_secs(3600),
            dependencies: vec![],
            additional_info: HashMap::new(),
        },
    };

    let task_clone = task.clone();
    mock_executor
        .expect_check_resource_availability()
        .with(eq(&task_clone.resources))
        .returning(|_| Ok(true));

    let build_engine = BuildEngine::new(
        Arc::new(mock_state_manager),
        Arc::new(mock_doc_engine),
        Arc::new(mock_executor),
        ResourceAllocation {
            cpu_cores: 4,
            memory_mb: 8192,
            disk_gb: 100,
        },
    );

    build_engine.check_resource_availability(&task).await?;
    Ok(())
}
