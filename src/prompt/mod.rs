use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod error;
pub mod generator;
pub mod storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: Option<String>,
    pub technologies: Vec<String>,
    pub project_type: ProjectType,
    pub language: String,
    pub framework: Option<String>,
    pub dependencies: Option<DependencyConfig>,
    pub build_config: Option<BuildConfig>,
    pub directory_structure: Option<HashMap<String, Vec<String>>>,
    pub initialization_commands: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub development: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build_system: String,
    pub build_steps: Vec<String>,
    pub test_command: Option<String>,
    pub package_manager: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Library,
    Application,
    Service,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub system_context: String,
    pub user_request: String,
}

impl Prompt {
    pub fn new(system_context: &str, user_request: &str) -> Self {
        Self {
            system_context: system_context.to_string(),
            user_request: user_request.to_string(),
        }
    }
}

#[async_trait::async_trait]
pub trait PromptProcessor: Send + Sync {
    async fn process_response(&self, response: String) -> Result<()>;
}

pub struct PromptManager {
    template_dir: String,
}

impl PromptManager {
    pub fn new(template_dir: &str) -> Result<Self> {
        let template_path = PathBuf::from(template_dir);
        std::fs::create_dir_all(&template_path)?;

        // Write task execution prompt template
        let task_prompt_path = template_path.join("task_execution_prompt.txt");
        std::fs::write(&task_prompt_path, include_str!("task_execution_prompt.txt"))?;

        Ok(Self {
            template_dir: template_dir.to_string(),
        })
    }

    pub async fn load_templates(&self) -> Result<HashMap<String, String>> {
        let mut templates = HashMap::new();
        let template_path = PathBuf::from(&self.template_dir);
        
        if template_path.exists() && template_path.is_dir() {
            let mut read_dir = tokio::fs::read_dir(template_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if let Some(name) = entry.path().file_stem() {
                    let template_content = tokio::fs::read_to_string(entry.path()).await?;
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
        let prompt_template = std::fs::read_to_string(prompt_template_path)?;

        // Construct the full prompt by combining the template with the user request
        let full_prompt = format!(
            "{}\n\n## User Request\n{}\n\n## Generate Configuration Based on Request",
            prompt_template, user_request
        );

        // Log the full prompt for debugging
        println!("Full Prompt for Project Config Generation:\n{}", full_prompt);

        // Use inference client to generate project configuration
        let inference_client = crate::inference::InferenceClient::new()?;
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
            let mut read_dir = tokio::fs::read_dir(template_path).await?;
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
        let re = regex::Regex::new(r"<response>(.*?)</response>").unwrap();
        if let Some(caps) = re.captures(response) {
            let response_text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            println!("Extracted response text: {:?}", response_text);
            if let Ok(project_config) = serde_json::from_str(response_text) {
                return Ok(project_config);
            }
        }

        println!("Attempting to find first JSON-like block");
        let json_re = regex::Regex::new(r"\{[^}]+\}").unwrap();
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

#[async_trait::async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, _response: String) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}
