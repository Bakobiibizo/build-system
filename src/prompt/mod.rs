use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::prompt::project_generation::{ProjectGenerationConfig, GenerationProjectType};
use reqwest;

pub mod error;
pub mod generator;
pub mod storage;
pub mod project_generation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: Option<String>,
    pub technologies: Vec<String>,
    pub project_type: ProjectType,
    pub language: String,
    pub framework: Option<String>,
    pub dependencies: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(rename = "build_system")]
    pub build_config: Option<BuildConfig>,
    pub directory_structure: Option<HashMap<String, Vec<String>>>,
    pub initialization_commands: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub build_tool: String,
    pub scripts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Library,
    Application,
    Service,
    Tool,
    #[serde(rename = "WebApplication")]
    WebApp,
    #[serde(rename = "CommandLineInterface")]
    Cli,
    #[serde(rename = "MicroService")]
    Microservice,
    #[serde(rename = "DesktopApplication")]
    Desktop,
    #[serde(rename = "MobileApplication")]
    Mobile,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Library => write!(f, "Library"),
            ProjectType::Application => write!(f, "Application"),
            ProjectType::Service => write!(f, "Service"),
            ProjectType::Tool => write!(f, "Tool"),
            ProjectType::WebApp => write!(f, "WebApplication"),
            ProjectType::Cli => write!(f, "CommandLineInterface"),
            ProjectType::Microservice => write!(f, "MicroService"),
            ProjectType::Desktop => write!(f, "DesktopApplication"),
            ProjectType::Mobile => write!(f, "MobileApplication"),
        }
    }
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

        // Write project generation prompt template
        let project_prompt_path = template_path.join("project_generation.txt");
        std::fs::write(&project_prompt_path, include_str!("project_generation.txt"))?;

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
        // Load project generation prompt template
        let template_path = PathBuf::from(&self.template_dir).join("project_generation.txt");
        let template = tokio::fs::read_to_string(template_path)
            .await
            .context("Failed to read project generation template")?;

        // Create prompt with user request
        let prompt = Prompt::new(&template, user_request);

        // Print full prompt for debugging
        println!("Full Prompt for Project Config Generation:");
        println!("{}", prompt.system_context);
        println!("\n## User Request");
        println!("{}", prompt.user_request);

        // Call LLM API
        let response = self.call_llm_api(&prompt).await?;

        // Parse response into project config
        self.parse_response(&response)
    }

    pub async fn list_templates(&self) -> Result<Vec<String>> {
        let mut templates = Vec::new();
        let template_path = PathBuf::from(&self.template_dir);
        
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

    fn parse_response(&self, response: &str) -> Result<ProjectConfig> {
        // Find the JSON part in the response
        let json_start = response.find('{')
            .ok_or_else(|| anyhow::anyhow!("No JSON object found in response"))?;
        let json_end = response.rfind('}')
            .ok_or_else(|| anyhow::anyhow!("No JSON object end found in response"))?;
        let json_str = &response[json_start..=json_end];

        // Parse the JSON into a ProjectGenerationConfig first
        let gen_config: ProjectGenerationConfig = serde_json::from_str(json_str)
            .context("Failed to parse response as ProjectGenerationConfig")?;

        // Convert to ProjectConfig
        Ok(ProjectConfig {
            name: gen_config.project_name,
            description: Some(gen_config.description),
            technologies: gen_config.technologies,
            project_type: match gen_config.project_type {
                GenerationProjectType::WebApplication => ProjectType::WebApp,
                GenerationProjectType::CommandLineInterface => ProjectType::Cli,
                GenerationProjectType::Library => ProjectType::Library,
                GenerationProjectType::MicroService => ProjectType::Microservice,
                GenerationProjectType::DesktopApplication => ProjectType::Desktop,
                GenerationProjectType::MobileApplication => ProjectType::Mobile,
            },
            language: gen_config.language,
            framework: Some(gen_config.framework),
            dependencies: Some({
                let mut deps = HashMap::new();
                if let Some(production) = gen_config.dependencies.get_dependencies("production") {
                    deps.insert("production".to_string(), production.clone());
                }
                if let Some(development) = gen_config.dependencies.get_dependencies("development") {
                    deps.insert("development".to_string(), development.clone());
                }
                deps
            }),
            build_config: Some(BuildConfig {
                build_tool: gen_config.build_config.build_tool,
                scripts: gen_config.build_config.scripts,
            }),
            directory_structure: Some(gen_config.directory_structure.into_iter()
                .map(|(k, v)| (k, v.to_vec()))
                .collect()),
            initialization_commands: Some(gen_config.initialization_commands),
            recommendations: Some(gen_config.recommendations),
        })
    }

    async fn call_llm_api(&self, prompt: &Prompt) -> Result<String> {
        // Call the LLM API to generate project configuration
        let client = reqwest::Client::new();
        let response = client
            .post("http://69.30.204.132:7099/v1/chat/completions")
            .header("Authorization", "Bearer sk-0123456789")
            .json(&serde_json::json!({
                "model": "llama3.2:latest",
                "messages": [
                    {
                        "role": "system",
                        "content": &prompt.system_context
                    },
                    {
                        "role": "user",
                        "content": &prompt.user_request
                    }
                ],
                "temperature": 0.7,
                "max_tokens": 2000
            }))
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        
        // Debug print the response
        println!("API Response: {}", serde_json::to_string_pretty(&response_json)?);
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get response content"))?;

        // Debug print the content
        println!("Content: {}", content);

        Ok(content.to_string())
    }
}

#[async_trait::async_trait]
impl PromptProcessor for PromptManager {
    async fn process_response(&self, _response: String) -> Result<()> {
        Ok(())
    }
}
