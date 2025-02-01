use serde::{Deserialize, Serialize};

/// Represents a generic prompt structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub complexity: u8,
}
