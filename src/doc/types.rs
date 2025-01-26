use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocType {
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json")]
    Json,
}

impl Default for DocType {
    fn default() -> Self {
        DocType::Markdown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DocumentationStepStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DocumentationStep {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub output: Option<String>,
    pub status: DocumentationStepStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Documentation {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub doc_type: DocType,
    pub path: PathBuf,
    pub project: String,
    pub priority: String,
    pub owner: String,
    pub tags: Vec<String>,
    pub additional_info: HashMap<String, String>,
    pub steps: Vec<DocumentationStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl std::fmt::Display for DocumentationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.description.as_ref().unwrap_or(&"".to_string()), self.status)
    }
}

impl std::fmt::Display for DocumentationStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentationStepStatus::Pending => write!(f, "Pending"),
            DocumentationStepStatus::InProgress => write!(f, "In Progress"),
            DocumentationStepStatus::Completed => write!(f, "Completed"),
            DocumentationStepStatus::Failed => write!(f, "Failed"),
        }
    }
}
