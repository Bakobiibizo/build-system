use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::collections::HashMap;

use crate::prompt::ProjectConfig;
use crate::tools::ExecutableTool;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectDesign {
    pub name: String,
    pub description: String,
    pub project_type: String,
    pub language: String,
    pub framework: String,
    pub technologies: Vec<String>,
    pub dependencies: DependencyConfig,
    pub build_config: BuildConfig,
    pub directory_structure: HashMap<String, Vec<String>>,
    pub initialization_commands: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyConfig {
    pub production: HashMap<String, String>,
    pub development: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub build_tool: String,
    pub scripts: BuildScripts,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildScripts {
    pub dev: String,
    pub build: String,
    pub test: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectGenerationError {
    #[error("IO error during project generation")]
    IoError(#[from] std::io::Error),
    #[error("Invalid project configuration: {0}")]
    ConfigurationError(String),
    #[error("Component generation error: {0}")]
    ComponentError(String),
    #[error("Build configuration error: {0}")]
    BuildError(String),
}

impl ProjectDesign {
    pub fn validate(&self) -> Result<(), ProjectGenerationError> {
        // Validate project name
        if self.name.trim().is_empty() {
            return Err(ProjectGenerationError::ConfigurationError(
                "Project name cannot be empty".to_string(),
            ));
        }

        // Validate required fields
        if self.language.trim().is_empty() {
            return Err(ProjectGenerationError::ConfigurationError(
                "Language must be specified".to_string(),
            ));
        }

        // Validate directory structure
        if self.directory_structure.is_empty() {
            return Err(ProjectGenerationError::ConfigurationError(
                "Directory structure must be defined".to_string(),
            ));
        }

        Ok(())
    }

    pub fn generate_project_structure(&self) -> Result<PathBuf, ProjectGenerationError> {
        // Validate configuration first
        self.validate()?;

        let project_root = PathBuf::from("build").join(&self.name);
        
        // Clean up existing directory if it exists
        if project_root.exists() {
            fs::remove_dir_all(&project_root).map_err(ProjectGenerationError::IoError)?;
        }

        // Create project root directory
        fs::create_dir_all(&project_root).map_err(ProjectGenerationError::IoError)?;

        // Generate core project files
        self.generate_readme(&project_root)?;
        self.generate_build_config(&project_root)?;
        self.generate_components(&project_root)?;

        Ok(project_root)
    }

    fn generate_build_config(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        match self.language.to_lowercase().as_str() {
            "rust" => self.generate_cargo_toml(project_root),
            "javascript" | "typescript" => self.generate_package_json(project_root),
            "python" => self.generate_requirements_txt(project_root),
            _ => Err(ProjectGenerationError::BuildError(format!(
                "Unsupported language: {}",
                self.language
            ))),
        }?;

        Ok(())
    }

    fn generate_components(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        for (component_name, files) in &self.directory_structure {
            let component_dir = project_root.join(component_name);
            fs::create_dir_all(&component_dir).map_err(ProjectGenerationError::IoError)?;

            for file in files {
                let file_path = component_dir.join(file);
                File::create(&file_path).map_err(ProjectGenerationError::IoError)?;
            }
        }

        Ok(())
    }

    fn generate_package_json(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let package_json = serde_json::json!({
            "name": self.name,
            "version": "0.1.0",
            "description": self.description,
            "scripts": {
                "dev": self.build_config.scripts.dev,
                "build": self.build_config.scripts.build,
                "test": self.build_config.scripts.test
            },
            "dependencies": self.dependencies.production,
            "devDependencies": self.dependencies.development
        });

        let file_path = project_root.join("package.json");
        let mut file = File::create(file_path).map_err(ProjectGenerationError::IoError)?;
        file.write_all(serde_json::to_string_pretty(&package_json).unwrap().as_bytes())
            .map_err(ProjectGenerationError::IoError)?;

        Ok(())
    }

    fn generate_requirements_txt(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let mut requirements = String::new();
        
        for (dep, version) in &self.dependencies.production {
            requirements.push_str(&format!("{}=={}\n", dep, version));
        }

        let file_path = project_root.join("requirements.txt");
        let mut file = File::create(file_path).map_err(ProjectGenerationError::IoError)?;
        file.write_all(requirements.as_bytes())
            .map_err(ProjectGenerationError::IoError)?;

        Ok(())
    }

    fn generate_readme(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let mut file = File::create(project_root.join("README.md")).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "# {}", self.name).map_err(ProjectGenerationError::IoError)?;
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "{}", self.description).map_err(ProjectGenerationError::IoError)?;
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Technologies").map_err(ProjectGenerationError::IoError)?;
        for tech in &self.technologies {
            writeln!(file, "- {}", tech).map_err(ProjectGenerationError::IoError)?;
        }
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Getting Started").map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "```bash").map_err(ProjectGenerationError::IoError)?;
        for cmd in &self.initialization_commands {
            writeln!(file, "{}", cmd).map_err(ProjectGenerationError::IoError)?;
        }
        writeln!(file, "```").map_err(ProjectGenerationError::IoError)?;

        Ok(())
    }

    fn generate_cargo_toml(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let mut file = File::create(project_root.join("Cargo.toml")).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "[package]").map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "name = \"{}\"", self.name).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "version = \"0.1.0\"").map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "edition = \"2021\"").map_err(ProjectGenerationError::IoError)?;
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        
        writeln!(file, "[dependencies]").map_err(ProjectGenerationError::IoError)?;
        for (name, version) in &self.dependencies.production {
            writeln!(file, "{} = \"{}\"", name, version).map_err(ProjectGenerationError::IoError)?;
        }
        
        if !self.dependencies.development.is_empty() {
            writeln!(file).map_err(ProjectGenerationError::IoError)?;
            writeln!(file, "[dev-dependencies]").map_err(ProjectGenerationError::IoError)?;
            for (name, version) in &self.dependencies.development {
                writeln!(file, "{} = \"{}\"", name, version).map_err(ProjectGenerationError::IoError)?;
            }
        }

        Ok(())
    }

    fn generate_architecture_md(&self, project_root: &Path) -> Result<(), ProjectGenerationError> {
        let mut file = File::create(project_root.join("docs").join("ARCHITECTURE.md")).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "# {} Architecture", self.name).map_err(ProjectGenerationError::IoError)?;
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Overview").map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "{}", self.description).map_err(ProjectGenerationError::IoError)?;
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Technology Stack").map_err(ProjectGenerationError::IoError)?;
        for tech in &self.technologies {
            writeln!(file, "- {}", tech).map_err(ProjectGenerationError::IoError)?;
        }
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Project Structure").map_err(ProjectGenerationError::IoError)?;
        for (dir, files) in &self.directory_structure {
            writeln!(file, "### {}/", dir).map_err(ProjectGenerationError::IoError)?;
            for filename in files {
                writeln!(file, "- {}", filename).map_err(ProjectGenerationError::IoError)?;
            }
        }
        writeln!(file).map_err(ProjectGenerationError::IoError)?;
        writeln!(file, "## Recommendations").map_err(ProjectGenerationError::IoError)?;
        for rec in &self.recommendations {
            writeln!(file, "- {}", rec).map_err(ProjectGenerationError::IoError)?;
        }

        Ok(())
    }
}

impl ExecutableTool for ProjectDesign {
    fn execute(&self, arguments: &str) -> Result<String, String> {
        // Parse the arguments as a JSON string representing project design
        let design: ProjectDesign = serde_json::from_str(arguments)
            .map_err(|e| format!("Failed to parse project design: {}", e))?;
        
        // Generate the project structure
        let project_path = design.generate_project_structure()
            .map_err(|e| format!("Failed to generate project structure: {}", e))?;
        
        // Return the project path as a string
        Ok(project_path.to_string_lossy().to_string())
    }
}

pub struct ProjectGenerator {
    config: ProjectConfig,
}

impl ProjectGenerator {
    pub fn new(config: ProjectConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self) -> Result<(), std::io::Error> {
        // Create project directory
        let project_root = PathBuf::from("build").join(&self.config.name);
        fs::create_dir_all(&project_root)?;

        // Create main script file based on language
        match self.config.language.to_lowercase().as_str() {
            "python" => {
                let main_script_path = project_root.join("main.py");
                let mut main_script = File::create(main_script_path)?;
                writeln!(main_script, "# Main script for {}", self.config.name)?;
                writeln!(main_script, "def main():")?;
                writeln!(main_script, "    print('Hello, {}!')", self.config.name)?;
                writeln!(main_script, "\nif __name__ == '__main__':")?;
                writeln!(main_script, "    main()")?;

                // Create requirements.txt
                let requirements_path = project_root.join("requirements.txt");
                File::create(requirements_path)?;
            }
            "rust" => {
                let main_script_path = project_root.join("src/main.rs");
                fs::create_dir_all(project_root.join("src"))?;
                let mut main_script = File::create(main_script_path)?;
                writeln!(main_script, "fn main() {{")?;
                writeln!(main_script, "    println!(\"Hello, {}!\");", self.config.name)?;
                writeln!(main_script, "}}")?;

                // Create Cargo.toml
                let cargo_path = project_root.join("Cargo.toml");
                let mut cargo = File::create(cargo_path)?;
                writeln!(cargo, "[package]")?;
                writeln!(cargo, "name = \"{}\"", self.config.name)?;
                writeln!(cargo, "version = \"0.1.0\"")?;
                writeln!(cargo, "edition = \"2021\"")?;
            }
            _ => {
                let main_script_path = project_root.join("main.txt");
                let mut main_script = File::create(main_script_path)?;
                writeln!(main_script, "Main script for {}", self.config.name)?;
            }
        }

        // Create README.md
        let readme_path = project_root.join("README.md");
        let mut readme = File::create(readme_path)?;
        writeln!(readme, "# {}", self.config.name)?;
        if let Some(desc) = &self.config.description {
            writeln!(readme, "\n{}", desc)?;
        }

        Ok(())
    }
}

pub fn parse_project_design(design_json: &str) -> Result<ProjectDesign, serde_json::Error> {
    serde_json::from_str(design_json)
}

pub fn generate_project(design_json: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let design = parse_project_design(design_json)?;
    design.generate_project_structure().map_err(|e| e.into())
}
