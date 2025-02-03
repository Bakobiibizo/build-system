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
use std::collections::HashMap;
use std::path::PathBuf;
use crate::project_generator::{ProjectDesign, Dependencies, BuildConfig};

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

pub struct BuildSystem;

impl BuildSystem {
    pub fn new() -> Self {
        BuildSystem
    }

    pub async fn generate_project(&self, config: crate::prompt::ProjectConfig) -> Result<()> {
        use crate::prompt::project_generation::DirectoryEntry;

        // Convert ProjectConfig to ProjectDesign
        let design = ProjectDesign {
            name: config.project_name,
            description: config.description,
            technologies: config.technologies,
            project_type: config.project_type.to_string(),
            language: config.language,
            framework: config.framework,
            dependencies: Dependencies {
                production: config.dependencies.production,
                development: config.dependencies.development,
            },
            build_config: BuildConfig {
                build_tool: config.build_config.build_tool,
                scripts: config.build_config.scripts,
            },
            directory_structure: config.directory_structure.into_iter()
                .map(|(k, v)| (k, v.to_vec()))
                .collect(),
        };

        let generator = project_generator::ProjectGenerator::new(design);
        generator.generate().await?;
        Ok(())
    }
}

pub struct ProjectManager;

impl ProjectManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_project(&self, config: &crate::prompt::ProjectConfig) -> Result<()> {
        // Convert ProjectConfig to ProjectDesign
        let design = ProjectDesign {
            name: config.project_name.clone(),
            description: config.description.clone(),
            technologies: config.technologies.clone(),
            project_type: config.project_type.to_string(),
            language: config.language.clone(),
            framework: config.framework.clone(),
            dependencies: Dependencies {
                production: config.dependencies.production.clone(),
                development: config.dependencies.development.clone(),
            },
            build_config: BuildConfig {
                build_tool: config.build_config.build_tool.clone(),
                scripts: config.build_config.scripts.clone(),
            },
            directory_structure: config.directory_structure.iter()
                .map(|(k, v)| (k.clone(), v.to_vec()))
                .collect(),
        };

        let generator = project_generator::ProjectGenerator::new(design);
        generator.generate().await?;
        Ok(())
    }
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
        generator.generate();

        assert!(fs::metadata("build/test").is_ok());

        Ok(())
    }
}

#[cfg(test)]
mod inference_test;
