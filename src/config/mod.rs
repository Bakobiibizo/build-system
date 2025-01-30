use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration management for the build system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Base directory for project generation
    pub base_project_dir: PathBuf,
    
    /// Default template directory
    pub template_dir: PathBuf,
    
    /// Logging configuration
    pub log_level: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            base_project_dir: PathBuf::from("build"),
            template_dir: PathBuf::from(".reference/templates"),
            log_level: "info".to_string(),
        }
    }
}
