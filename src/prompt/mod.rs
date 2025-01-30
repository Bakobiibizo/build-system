use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::fs as tokio_fs;
use regex::Regex;
use chrono;
use std::path::PathBuf;

use crate::inference::InferenceClient;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectConfig {
    pub project_name: String,
    pub description: Option<String>,
    pub technologies: Vec<String>,
    pub project_type: Option<ProjectType>,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub architecture: Option<String>,
    pub dependencies: Option<DependencyConfig>,
    pub build_config: Option<BuildConfig>,
    pub directory_structure: Option<HashMap<String, Vec<String>>>,
    pub initialization_commands: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
    pub additional_config: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProjectType {
    WebApplication,
    CommandLineInterface,
    Library,
    MicroService,
    DesktopApplication,
    MobileApplication,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyConfig {
    pub production: HashMap<String, String>,
    pub development: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildConfig {
    pub build_tool: String,
    pub scripts: BuildScripts,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildScripts {
    pub dev: String,
    pub build: String,
    pub test: String,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        Self {
            project_name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_technologies(mut self, technologies: Vec<String>) -> Self {
        self.technologies = technologies;
        self
    }

    pub fn with_project_type(mut self, project_type: ProjectType) -> Self {
        self.project_type = Some(project_type);
        self
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = Some(language.to_string());
        self
    }

    pub fn with_framework(mut self, framework: &str) -> Self {
        self.framework = Some(framework.to_string());
        self
    }

    pub fn with_dependencies(mut self, dependencies: DependencyConfig) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    pub fn with_build_config(mut self, build_config: BuildConfig) -> Self {
        self.build_config = Some(build_config);
        self
    }

    pub fn with_directory_structure(mut self, directory_structure: HashMap<String, Vec<String>>) -> Self {
        self.directory_structure = Some(directory_structure);
        self
    }

    pub fn with_initialization_commands(mut self, initialization_commands: Vec<String>) -> Self {
        self.initialization_commands = Some(initialization_commands);
        self
    }

    pub fn with_recommendations(mut self, recommendations: Vec<String>) -> Self {
        self.recommendations = Some(recommendations);
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromptProjectConfig {
    pub name: String,
    pub description: Option<String>,
    pub technologies: Option<Vec<String>>,
    // Add other fields as needed
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectGenerationConfig {
    pub project_name: String,
    pub project_type: ProjectType,
    pub language: String,
    pub framework: String,
    pub dependencies: DependencyConfig,
    pub build_config: BuildConfig,
    pub directory_structure: HashMap<String, Vec<String>>,
    pub initialization_commands: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prompt {
    pub system_context: String,
    pub user_request: String,
    pub build_context: Option<String>,
}

impl Prompt {
    pub fn new(system_context: &str, user_request: &str) -> Self {
        Self {
            system_context: system_context.to_string(),
            user_request: user_request.to_string(),
            build_context: None,
        }
    }
}

#[async_trait]
pub trait PromptProcessor: Send + Sync {
    async fn process_response(&self, response: String) -> Result<()>;
}

pub struct PromptManager {
    template_dir: String,
}

impl PromptManager {
    pub fn new(template_dir: &str) -> Result<Self> {
        let template_path = PathBuf::from(template_dir);
        fs::create_dir_all(&template_path)?;

        // Write task execution prompt template
        let task_prompt_path = template_path.join("task_execution_prompt.txt");
        fs::write(&task_prompt_path, include_str!("task_execution_prompt.txt"))?;

        Ok(Self {
            template_dir: template_dir.to_string(),
        })
    }

    pub async fn load_templates(&self) -> Result<HashMap<String, String>> {
        let mut templates = HashMap::new();
        let template_path = PathBuf::from(&self.template_dir);
        
        if template_path.exists() && template_path.is_dir() {
            let mut read_dir = tokio_fs::read_dir(template_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if let Some(name) = entry.path().file_stem() {
                    let template_content = tokio_fs::read_to_string(entry.path()).await?;
                    templates.insert(
                        name.to_string_lossy().to_string(), 
                        template_content
                    );
                }
            }
        }
        
        Ok(templates)
    }

    pub async fn generate_project_config(&self, user_request: &str) -> Result<ProjectConfig> {
        // Read the task execution prompt template
        let prompt_template_path = PathBuf::from(&self.template_dir).join("task_execution_prompt.txt");
        let prompt_template = fs::read_to_string(prompt_template_path)?;

        // Construct the full prompt by combining the template with the user request
        let full_prompt = format!(
            "{}\n\n## User Request\n{}\n\n## Generate Configuration Based on Request",
            prompt_template, user_request
        );

        // Log the full prompt for debugging
        println!("Full Prompt for Project Config Generation:\n{}", full_prompt);

        // Use inference client to generate project configuration
        let inference_client = InferenceClient::new()?;
        let generated_config_json = inference_client.generate_project_config(&full_prompt).await?;

        // Log the generated JSON for debugging
        println!("Generated Project Config JSON:\n{}", generated_config_json);

        // Parse the generated JSON into ProjectConfig
        let project_config: ProjectConfig = serde_json::from_str(&generated_config_json)
            .context("Failed to parse generated project configuration")?;

        Ok(project_config)
    }

    pub async fn list_templates(&self) -> Result<Vec<String>> {
        let template_path = PathBuf::from(&self.template_dir);
        let mut templates = Vec::new();

        if template_path.exists() && template_path.is_dir() {
            let mut read_dir = tokio_fs::read_dir(template_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if let Some(name) = entry.path().file_stem() {
                    templates.push(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(templates)
    }

    pub async fn parse_response(&self, response: &str) -> Result<ProjectConfig> {
        // Log the raw response
        println!("Raw response received: {:?}", response);

        // Ensure .reference/ai_responses directory exists
        let save_dir = PathBuf::from(".reference/ai_responses");
        std::fs::create_dir_all(&save_dir).map_err(|e| {
            eprintln!("Failed to create response save directory: {}", e);
            anyhow::anyhow!("Failed to create response save directory: {}", e)
        })?;

        // Generate unique filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let save_path = save_dir.join(format!("project_config_response_{}.txt", timestamp));
        
        // Save the full response
        std::fs::write(&save_path, response).map_err(|e| {
            eprintln!("Failed to save response to {}: {}", save_path.display(), e);
            anyhow::anyhow!("Failed to save response to {}: {}", save_path.display(), e)
        })?;

        // Attempt parsing methods with detailed logging
        println!("Attempting to parse entire response as JSON");
        if let Ok(project_config) = serde_json::from_str(response) {
            return Ok(project_config);
        }

        println!("Attempting to extract JSON from response tags");
        let re = Regex::new(r"<response>(.*?)</response>").unwrap();
        if let Some(caps) = re.captures(response) {
            let response_text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            println!("Extracted response text: {:?}", response_text);
            if let Ok(project_config) = serde_json::from_str(response_text) {
                return Ok(project_config);
            }
        }

        println!("Attempting to find first JSON-like block");
        let json_re = Regex::new(r"\{[^}]+\}").unwrap();
        if let Some(json_match) = json_re.find(response) {
            let json_text = json_match.as_str();
            println!("Found JSON-like block: {:?}", json_text);
            if let Ok(project_config) = serde_json::from_str(json_text) {
                return Ok(project_config);
            }
        }

        // If all parsing attempts fail, return an error with the saved file path
        Err(anyhow::anyhow!(
            "Failed to parse project configuration from response. Raw response saved to {}",
            save_path.display()
        ))
    }
}

#[async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, _response: String) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}

pub struct PromptGenerator;

impl PromptGenerator {
    pub fn generate_project_prompt(config: &ProjectConfig) -> Prompt {
        Prompt::new(
            "Generate a project configuration based on user requirements", 
            &format!("Create a project named {} with description: {}", 
                config.project_name, 
                config.description.clone().unwrap_or_default()
            )
        )
    }
}
