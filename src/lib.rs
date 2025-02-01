// Core modules
pub mod cli;
pub mod doc;
pub mod inference;
pub mod project_generator;
pub mod prompt;
pub mod state;
pub mod tools;
pub mod build;
pub mod validation;

// Utility and support modules
pub mod config;
pub mod logging;

// AI and inference modules
pub mod llm;

// Optional feature modules
#[cfg(feature = "web-features")]
pub mod web;

#[cfg(feature = "ai-features")]
pub mod ai;

pub mod error;

// Exports
pub use cli::handle_cli_command;
pub use validation::BuildValidation;
pub use prompt::storage::{PromptStorage, Storage};
pub use state::manager::StateManager;
pub use state::types::{TaskId, TaskState, TaskStatus, TaskMetadata};
pub use build::error::BuildError;
pub use prompt::generator::PromptGenerator;

use anyhow::Result;
use std::path::PathBuf;

/// Save model output and build files for validation
pub async fn save_model_output_for_validation(
    build_path: PathBuf,
    model_response: String,
    storage_path: PathBuf,
) -> Result<()> {
    // Initialize storage
    let storage = prompt::storage::Storage::new(storage_path)?;

    // Capture the build output and model response
    let validation = validation::capture_build_output(build_path, model_response)?;

    // Save the validation data
    validation.save(&storage)?;

    Ok(())
}

/// Validate a previously saved build
pub async fn validate_saved_build(
    storage_path: PathBuf,
    validation_key: &str,
) -> Result<validation::ValidationReport> {
    // Initialize storage
    let storage = prompt::storage::Storage::new(storage_path)?;

    // Load the validation data
    let validation = BuildValidation::load(&storage, validation_key)?
        .ok_or_else(|| anyhow::anyhow!("Validation data not found for key: {}", validation_key))?;

    // Run validation
    validation::validate_build(&validation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::ProjectConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_project_config_serialization() -> Result<()> {
        let config = ProjectConfig {
            name: "test".to_string(),
            description: Some("Test project".to_string()),
            technologies: vec!["rust".to_string()],
            project_type: prompt::ProjectType::Application,
            language: "rust".to_string(),
            framework: Some("actix-web".to_string()),
            dependencies: None,
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        let json = serde_json::to_string(&config)?;
        let deserialized: ProjectConfig = serde_json::from_str(&json)?;

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.description, deserialized.description);

        Ok(())
    }

    #[test]
    fn test_project_generation() -> Result<()> {
        let _temp_dir = TempDir::new()?;
        let config = ProjectConfig {
            name: "test".to_string(),
            description: Some("Test project".to_string()),
            technologies: vec!["rust".to_string()],
            project_type: prompt::ProjectType::Application,
            language: "rust".to_string(),
            framework: Some("actix-web".to_string()),
            dependencies: None,
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        let generator = project_generator::ProjectGenerator::new(config);
        generator.generate()?;

        assert!(fs::metadata("build/test").is_ok());

        Ok(())
    }
}

#[cfg(test)]
mod inference_test;
