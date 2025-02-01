use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs;
use async_trait::async_trait;
use crate::tools::ExecutableTool;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDesign {
    pub name: String,
    pub description: String,
    pub technologies: Vec<String>,
    pub project_type: String,
    pub language: String,
    pub framework: String,
    pub dependencies: Dependencies,
    pub build_config: BuildConfig,
    pub directory_structure: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependencies {
    pub production: HashMap<String, String>,
    pub development: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build_tool: String,
    pub scripts: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProjectGenerationError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    ValidationError(String),
}

impl From<std::io::Error> for ProjectGenerationError {
    fn from(err: std::io::Error) -> Self {
        ProjectGenerationError::IoError(err)
    }
}

impl From<serde_json::Error> for ProjectGenerationError {
    fn from(err: serde_json::Error) -> Self {
        ProjectGenerationError::SerializationError(err)
    }
}

impl std::fmt::Display for ProjectGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectGenerationError::IoError(e) => write!(f, "IO error: {}", e),
            ProjectGenerationError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ProjectGenerationError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ProjectGenerationError {}

impl ProjectDesign {
    pub fn validate(&self) -> Result<(), ProjectGenerationError> {
        if self.name.is_empty() {
            return Err(ProjectGenerationError::ValidationError(
                "Project name cannot be empty".to_string(),
            ));
        }

        if self.language.is_empty() {
            return Err(ProjectGenerationError::ValidationError(
                "Programming language cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn generate_project_structure(&self) -> Result<(), ProjectGenerationError> {
        let project_root = format!("build/{}", self.name);
        fs::create_dir_all(&project_root).await?;

        // Create directory structure
        for (dir, files) in &self.directory_structure {
            let dir_path = format!("{}/{}", project_root, dir);
            fs::create_dir_all(&dir_path).await?;
            
            for file in files {
                let file_path = format!("{}/{}", dir_path, file);
                fs::write(&file_path, "").await?;
            }
        }

        // Create dependency files
        let requirements = self.dependencies.production.get("package")
            .map(|deps| deps.to_string())
            .unwrap_or_default();
        
        let dev_requirements = self.dependencies.development.get("package")
            .map(|deps| deps.to_string())
            .unwrap_or_default();
        
        fs::write(format!("{}/requirements.txt", project_root), requirements).await?;
        fs::write(format!("{}/dev-requirements.txt", project_root), dev_requirements).await?;

        // Create build.json
        let build_json = serde_json::to_string_pretty(&self.build_config)?;
        fs::write(format!("{}/build.json", project_root), build_json).await?;

        // Generate architecture.md
        self.generate_architecture_md(Path::new(&project_root)).await?;

        Ok(())
    }

    async fn generate_architecture_md(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let mut content = format!(
            "# {} Architecture\n\n## Overview\n{}\n\n",
            self.name, self.description
        );

        content.push_str("## Technologies\n");
        for tech in &self.technologies {
            content.push_str(&format!("- {}\n", tech));
        }

        content.push_str("\n## Project Structure\n");
        for (dir, files) in &self.directory_structure {
            content.push_str(&format!("\n### {}/\n", dir));
            for file in files {
                content.push_str(&format!("- {}\n", file));
            }
        }

        content.push_str("\n## Dependencies\n");
        content.push_str("\n### Production Dependencies\n");
        for (name, version) in &self.dependencies.production {
            content.push_str(&format!("- {} v{}\n", name, version));
        }

        content.push_str("\n### Development Dependencies\n");
        for (name, version) in &self.dependencies.development {
            content.push_str(&format!("- {} v{}\n", name, version));
        }

        content.push_str("\n## Build and Development\n");
        content.push_str(&format!("Build tool: {}\n\n", self.build_config.build_tool));
        content.push_str("Available scripts:\n");
        for (name, script) in &self.build_config.scripts {
            content.push_str(&format!("- {}: `{}`\n", name, script));
        }

        fs::write(project_root.join("architecture.md"), content).await?;
        Ok(())
    }
}

#[async_trait]
impl ExecutableTool for ProjectDesign {
    async fn execute(&self, _arguments: &str) -> Result<String, String> {
        self.validate()
            .map_err(|e| format!("Project validation failed: {}", e))?;
        
        self.generate_project_structure()
            .await
            .map_err(|e| format!("Project generation failed: {}", e))?;
        
        Ok(format!("Project '{}' generated successfully", self.name))
    }

    fn get_tool_definition(&self) -> crate::tools::Tool {
        crate::tools::Tool {
            name: "project".to_string(),
            description: "Generate a new project using a project design".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the project"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "description": {
                        "type": "string",
                        "description": "Project description"
                    }
                },
                "required": ["name", "language"]
            }),
        }
    }

    fn get_short_description(&self) -> String {
        "Generate a new project using a project design".to_string()
    }

    fn get_long_description(&self) -> String {
        r#"Generates a new project based on a project design specification.
The project design includes:
- Project metadata (name, description, etc.)
- Technologies and frameworks
- Dependencies (production and development)
- Build configuration
- Directory structure"#.to_string()
    }
}

pub fn parse_project_design(json: &str) -> Result<ProjectDesign, ProjectGenerationError> {
    serde_json::from_str(json).map_err(ProjectGenerationError::SerializationError)
}

pub struct ProjectGenerator {
    config: ProjectDesign,
}

impl ProjectGenerator {
    pub fn new(config: ProjectDesign) -> Self {
        Self { config }
    }

    pub async fn generate(&self) -> Result<(), ProjectGenerationError> {
        self.config.generate_project_structure().await
    }
}
