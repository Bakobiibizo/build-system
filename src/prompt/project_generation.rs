use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// Represents a comprehensive project generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGenerationConfig {
    /// The name of the project
    #[serde(alias = "name")]
    pub project_name: String,

    /// Project description
    #[serde(default)]
    pub description: String,

    /// Primary programming language
    pub language: String,

    /// Web framework or library
    #[serde(default)]
    pub framework: String,

    /// Type of project
    pub project_type: GenerationProjectType,

    /// List of technologies used
    #[serde(default)]
    pub technologies: Vec<String>,

    /// Project components and their responsibilities
    #[serde(default)]
    pub components: HashMap<String, String>,

    /// Detailed directory structure
    #[serde(default)]
    pub directory_structure: HashMap<String, DirectoryEntry>,

    /// Production and development dependencies
    #[serde(default)]
    pub dependencies: GenerationDependencyConfig,

    /// Build and configuration details
    #[serde(rename = "build_system", default)]
    pub build_config: GenerationBuildConfig,

    /// Commands to initialize the project
    #[serde(default)]
    pub initialization_commands: Vec<String>,

    /// Additional recommendations
    #[serde(default)]
    pub recommendations: Vec<String>,
}

/// Represents different types of software projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationProjectType {
    #[serde(rename = "WebApplication")]
    WebApplication,
    #[serde(rename = "CommandLineInterface")]
    CommandLineInterface,
    Library,
    #[serde(rename = "MicroService")]
    MicroService,
    #[serde(rename = "DesktopApplication")]
    DesktopApplication,
    #[serde(rename = "MobileApplication")]
    MobileApplication,
    Application,
    Service,
    Tool,
}

impl std::fmt::Display for GenerationProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationProjectType::WebApplication => write!(f, "WebApplication"),
            GenerationProjectType::CommandLineInterface => write!(f, "CommandLineInterface"),
            GenerationProjectType::Library => write!(f, "Library"),
            GenerationProjectType::MicroService => write!(f, "MicroService"),
            GenerationProjectType::DesktopApplication => write!(f, "DesktopApplication"),
            GenerationProjectType::MobileApplication => write!(f, "MobileApplication"),
            GenerationProjectType::Application => write!(f, "Application"),
            GenerationProjectType::Service => write!(f, "Service"),
            GenerationProjectType::Tool => write!(f, "Tool"),
        }
    }
}

/// Dependency configuration for both production and development
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationDependencyConfig {
    pub production: HashMap<String, String>,
    pub development: HashMap<String, String>,
}

impl GenerationDependencyConfig {
    pub fn get_dependencies(&self, env: &str) -> Option<&HashMap<String, String>> {
        match env {
            "production" => Some(&self.production),
            "development" => Some(&self.development),
            _ => None,
        }
    }

    pub fn new() -> Self {
        Self {
            production: HashMap::new(),
            development: HashMap::new(),
        }
    }
}

/// Build and configuration details
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationBuildConfig {
    pub build_tool: String,
    pub scripts: HashMap<String, String>,
}

/// Directory entry that can be either a single file or a list of files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DirectoryEntry {
    Files(Vec<String>),
    File(String),
}

impl DirectoryEntry {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            DirectoryEntry::Files(files) => files.clone(),
            DirectoryEntry::File(file) => vec![file.clone()],
        }
    }
}

impl ProjectGenerationConfig {
    /// Create a new project generation configuration
    pub fn new(
        project_name: String,
        description: String,
        language: String,
        framework: String,
        project_type: GenerationProjectType,
    ) -> Result<Self, String> {
        if !is_valid_project_name(&project_name) {
            return Err("Invalid project name. Use kebab-case (e.g., my-project)".to_string());
        }

        Ok(Self {
            project_name,
            description,
            language,
            framework,
            project_type,
            technologies: Vec::new(),
            components: HashMap::new(),
            directory_structure: HashMap::new(),
            dependencies: GenerationDependencyConfig::new(),
            build_config: GenerationBuildConfig::default(),
            initialization_commands: Vec::new(),
            recommendations: Vec::new(),
        })
    }

    /// Validate the project generation configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check required fields
        if self.project_name.is_empty() {
            return Err("Project name is required".to_string());
        }
        if !is_valid_project_name(&self.project_name) {
            return Err("Invalid project name format".to_string());
        }
        if self.description.is_empty() {
            return Err("Project description is required".to_string());
        }
        if self.language.is_empty() {
            return Err("Programming language is required".to_string());
        }
        if self.framework.is_empty() {
            return Err("Framework is required".to_string());
        }

        // Check directory structure
        for (dir, _) in &self.directory_structure {
            if dir.is_empty() {
                return Err("Directory name cannot be empty".to_string());
            }
            if dir.contains('/') || dir.contains('\\') {
                return Err("Directory name cannot contain path separators".to_string());
            }
        }

        Ok(())
    }

    /// Add a production dependency
    pub fn add_production_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.production.insert(name.to_string(), version.to_string());
    }

    /// Add a development dependency
    pub fn add_development_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.development.insert(name.to_string(), version.to_string());
    }

    /// Set build scripts
    pub fn set_build_scripts(&mut self, dev: &str, build: &str, test: &str) -> Result<(), String> {
        if dev.is_empty() || build.is_empty() || test.is_empty() {
            return Err("Build scripts cannot be empty".to_string());
        }

        self.build_config.scripts.insert("dev".to_string(), dev.to_string());
        self.build_config.scripts.insert("build".to_string(), build.to_string());
        self.build_config.scripts.insert("test".to_string(), test.to_string());

        Ok(())
    }

    /// Add initialization command
    pub fn add_initialization_command(&mut self, command: &str) {
        self.initialization_commands.push(command.to_string());
    }

    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: &str) {
        self.recommendations.push(recommendation.to_string());
    }

    /// Add a component with its responsibility
    pub fn add_component(&mut self, name: &str, responsibility: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Component name cannot be empty".to_string());
        }
        if responsibility.is_empty() {
            return Err("Component responsibility cannot be empty".to_string());
        }

        // Validate component name format
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Component name can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }

        self.components.insert(name.to_string(), responsibility.to_string());
        Ok(())
    }

    /// Add a technology
    pub fn add_technology(&mut self, technology: &str) -> Result<(), String> {
        if technology.is_empty() {
            return Err("Technology name cannot be empty".to_string());
        }
        self.technologies.push(technology.to_string());
        Ok(())
    }

    /// Generate a sample project configuration for testing
    pub fn sample_web_project() -> Self {
        let mut config = Self::new(
            "sample-web-app".to_string(),
            "A sample web application".to_string(),
            "Python".to_string(),
            "Flask".to_string(),
            GenerationProjectType::WebApplication,
        ).unwrap();

        config.add_technology("Flask").unwrap();
        config.add_technology("SQLite").unwrap();
        config.add_technology("JWT").unwrap();

        config.add_production_dependency("flask", "2.0.1");
        config.add_production_dependency("sqlalchemy", "1.4.23");
        config.add_development_dependency("pytest", "6.2.5");
        config.add_development_dependency("black", "21.9b0");

        config.set_build_scripts(
            "flask run",
            "python setup.py build",
            "pytest",
        ).unwrap();

        config.add_initialization_command("python -m venv venv");
        config.add_initialization_command("source venv/bin/activate");
        config.add_initialization_command("pip install -r requirements.txt");

        config.add_recommendation("Use environment variables for configuration");
        config.add_recommendation("Implement comprehensive error handling");

        config
    }
}

fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_generation_config_creation() {
        let config = ProjectGenerationConfig::new(
            "test-project".to_string(),
            "A test project".to_string(),
            "Python".to_string(),
            "Flask".to_string(),
            GenerationProjectType::WebApplication,
        ).unwrap();

        assert_eq!(config.project_name, "test-project");
        assert_eq!(config.language, "Python");
        assert_eq!(config.framework, "Flask");
        assert!(config.technologies.is_empty());
        assert!(config.components.is_empty());
        assert!(config.directory_structure.is_empty());
    }
}
