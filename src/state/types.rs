use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new(id: &str) -> Self {
        TaskId(id.to_string())
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TaskId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TaskId::new(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub name: String,
    pub description: Option<String>,
    pub owner: String,
    pub dependencies: Vec<TaskId>,
    pub estimated_duration: Duration,
    pub priority: i32,
    pub tags: Vec<String>,
    pub additional_info: HashMap<String, String>,
}

impl Default for TaskMetadata {
    fn default() -> Self {
        TaskMetadata {
            name: String::new(),
            description: None,
            owner: String::new(),
            dependencies: Vec::new(),
            estimated_duration: Duration::from_secs(0),
            priority: 0,
            tags: Vec::new(),
            additional_info: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskState {
    pub id: TaskId,
    pub status: TaskStatus,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskState {
    pub fn new(id: TaskId) -> Self {
        let now = Utc::now();
        TaskState {
            id,
            status: TaskStatus::Pending,
            metadata: TaskMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for TaskState {
    fn default() -> Self {
        let id = TaskId::new("default");
        Self::new(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tasks: HashMap<TaskId, TaskState>,
    pub timestamp: DateTime<Utc>,
}
