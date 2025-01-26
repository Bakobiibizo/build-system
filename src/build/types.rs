use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::state::TaskStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraint {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu: ResourceConstraint,
    pub memory: ResourceConstraint,
    pub disk: ResourceConstraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub content: String,
    pub is_executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub name: String,
    pub description: Option<String>,
    pub owner: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub estimated_duration: Duration,
    pub dependencies: Vec<String>,
    pub additional_info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTask {
    pub id: String,
    pub resources: ResourceRequirements,
    pub changes: Vec<FileChange>,
    pub metadata: TaskMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub disk_gb: u64,
}

#[async_trait]
pub trait BuildExecutor: Send + Sync {
    async fn execute_task(&self, task: BuildTask) -> Result<(), crate::build::error::BuildError>;
    async fn get_task_status(&self, id: &str) -> Result<TaskStatus, crate::build::error::BuildError>;
    async fn cancel_task(&self, id: &str) -> Result<(), crate::build::error::BuildError>;
    async fn apply_changes(&self, changes: &[FileChange]) -> Result<(), crate::build::error::BuildError>;
    async fn check_resource_availability(&self, requirements: &ResourceRequirements) -> Result<bool, crate::build::error::BuildError>;
}
