use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::prompt::project_generation::{ProjectGenerationConfig, GenerationProjectType, GenerationBuildConfig, DirectoryEntry};
use reqwest;

pub mod error;
pub mod generator;
pub mod storage;
pub mod project_generation;

// Re-export the main types
pub use project_generation::{ProjectGenerationConfig as ProjectConfig, GenerationProjectType as ProjectType};

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

#[derive(Debug)]
pub struct PromptManager {
    template_dir: PathBuf,
    templates: HashMap<String, String>,
}

impl PromptManager {
    pub fn new(template_dir: &str) -> Result<Self> {
        let template_path = PathBuf::from(template_dir);
        std::fs::create_dir_all(&template_path)?;

        // Write the project generation prompt template
        let project_prompt_path = template_path.join("project_generation.txt");
        std::fs::write(&project_prompt_path, include_str!("project_generation_prompt.md"))?;

        Ok(Self {
            template_dir: template_path,
            templates: HashMap::new(),
        })
    }

    pub async fn load_templates(&mut self) -> Result<()> {
        let mut templates = HashMap::new();
        let template_path = &self.template_dir;
        
        if template_path.exists() && template_path.is_dir() {
            let mut read_dir = tokio::fs::read_dir(template_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if let Ok(content) = tokio::fs::read_to_string(entry.path()).await {
                    if let Some(name) = entry.file_name().to_str() {
                        templates.insert(name.to_string(), content);
                    }
                }
            }
        }
        
        self.templates = templates;
        Ok(())
    }

    pub async fn generate_project_config(&self, user_request: &str) -> Result<ProjectConfig> {
        let template_path = self.template_dir.join("project_generation.txt");
        let template = tokio::fs::read_to_string(template_path)
            .await
            .context("Failed to read project generation template")?;

        let prompt = Prompt::new(&template, user_request);
        let response = self.call_llm_api(&prompt).await?;
        self.parse_response(&response)
    }

    pub async fn list_templates(&self) -> Result<Vec<String>> {
        let mut templates = Vec::new();
        let template_path = &self.template_dir;
        
        if template_path.exists() && template_path.is_dir() {
            let mut read_dir = tokio::fs::read_dir(template_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                if let Some(name) = entry.file_name().to_str() {
                    templates.push(name.to_string());
                }
            }
        }
        
        Ok(templates)
    }

    fn parse_response(&self, response: &str) -> Result<ProjectConfig> {
        // Find the JSON object in the response
        let json_start = response
            .find('{')
            .ok_or_else(|| anyhow::anyhow!("No JSON object start found in response"))?;
        let json_end = response
            .rfind('}')
            .ok_or_else(|| anyhow::anyhow!("No JSON object end found in response"))?;
        let json_str = &response[json_start..=json_end];

        // Parse the JSON into a ProjectGenerationConfig
        let gen_config: ProjectGenerationConfig = serde_json::from_str(json_str)
            .context("Failed to parse response as ProjectGenerationConfig")?;

        Ok(gen_config)
    }

    async fn call_llm_api(&self, prompt: &Prompt) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&prompt)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

#[async_trait::async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, _response: String) -> Result<()> {
        Ok(())
    }
}
