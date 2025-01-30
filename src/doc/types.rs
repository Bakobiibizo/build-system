use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use std::str::FromStr;
use clap::ValueEnum;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum DocType {
    /// Project overview documentation
    ProjectOverview,
    /// Technical specification
    TechnicalSpec,
    /// Architecture design
    Architecture,
    /// API documentation
    Api,
    /// User manual
    UserManual,
    /// JSON configuration
    Json,
    /// Markdown documentation
    Markdown,
    /// Other documentation type
    Other,
}

impl FromStr for DocType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project_overview" | "projectoverview" => Ok(DocType::ProjectOverview),
            "technical_spec" | "technicalspec" => Ok(DocType::TechnicalSpec),
            "architecture" => Ok(DocType::Architecture),
            "api" => Ok(DocType::Api),
            "user_manual" | "usermanual" => Ok(DocType::UserManual),
            "json" => Ok(DocType::Json),
            "markdown" | "md" => Ok(DocType::Markdown),
            "other" => Ok(DocType::Other),
            _ => Err(format!("Unknown documentation type: {}", s)),
        }
    }
}

impl DocType {
    /// Get a string representation of the documentation type
    pub fn as_str(&self) -> &'static str {
        match self {
            DocType::ProjectOverview => "project_overview",
            DocType::TechnicalSpec => "technical_spec",
            DocType::Architecture => "architecture",
            DocType::Api => "api",
            DocType::UserManual => "user_manual",
            DocType::Json => "json",
            DocType::Markdown => "markdown",
            DocType::Other => "other",
        }
    }
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
    #[serde(default)]
    pub metadata: HashMap<String, String>,
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

impl Documentation {
    pub fn new(
        title: String,
        content: String,
        doc_type: DocType,
        path: PathBuf,
        project: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            content,
            doc_type,
            path,
            project,
            owner: String::new(),
            priority: String::new(),
            tags: Vec::new(),
            additional_info: HashMap::new(),
            steps: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}
