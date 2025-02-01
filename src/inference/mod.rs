use async_openai::{
    config::OpenAIConfig,
    types::Role,
};
use anyhow::{Context, Result, anyhow};
use serde_json::json;
use std::path::PathBuf;

use crate::prompt::Prompt;
use crate::state::types::TaskId;
use crate::state::StateManager;
use crate::build::BuildManager;

#[derive(Clone)]
pub struct OpenAIConfigWrapper(OpenAIConfig);

impl OpenAIConfigWrapper {
    pub fn new(config: OpenAIConfig) -> Self {
        Self(config)
    }

    pub fn inner(&self) -> &OpenAIConfig {
        &self.0
    }
}

pub struct InferenceClient {
    api_key: String,
    base_url: String,
    model: String,
}

impl InferenceClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("INFERENCE_API_KEY")
            .map_err(|_| anyhow!("INFERENCE_API_KEY environment variable not found"))?;
        let base_url = std::env::var("INFERENCE_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("INFERENCE_API_MODEL")
            .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        println!("Using inference model: {}", model);
        println!("Using base URL: {}", base_url);

        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }

    pub async fn execute_task_prompt(&self, prompt: &Prompt, _task_id: &TaskId) -> Result<String> {
        // Create OpenAI API request
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": Role::System,
                    "content": &prompt.system_context
                },
                {
                    "role": Role::User,
                    "content": &prompt.user_request
                }
            ],
            "temperature": 0.7
        });

        // Send request to OpenAI API
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Extract response content
        response.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Failed to extract content from OpenAI response"))
    }

    pub async fn generate_project_config(&self, prompt: &str) -> Result<String> {
        // Read the project generation prompt template
        let template_path = std::path::Path::new("src/prompt/templates/project_generation_prompt.md");
        let system_prompt = std::fs::read_to_string(template_path)
            .context("Failed to read project generation prompt template")?;

        // Get temperature from env or use default
        let temperature = std::env::var("INFERENCE_API_TEMPERATURE")
            .ok()
            .and_then(|t| t.parse::<f32>().ok())
            .unwrap_or(0.7);

        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": Role::System,
                    "content": system_prompt
                },
                {
                    "role": Role::User,
                    "content": prompt
                }
            ],
            "temperature": temperature
        });

        println!("Sending request to: {}/chat/completions", self.base_url);
        
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        println!("Response status: {}", response.status());

        let response_json = response.json::<serde_json::Value>().await?;
        
        // Extract the content from the response
        let content = response_json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| {
                let err_msg = format!("Failed to extract content from response. Response structure: {:?}", response_json);
                anyhow!(err_msg)
            })?;

        // Try to find JSON in the content
        if let Some(json_str) = Self::extract_json_from_content(content) {
            // Parse the JSON
            let config: serde_json::Value = serde_json::from_str(json_str)
                .context("Failed to parse extracted JSON")?;
            
            let obj = config.as_object()
                .ok_or_else(|| anyhow!("JSON must be an object"))?;
            
            // Check for keys containing required terms
            let has_name = obj.keys()
                .any(|k| k.to_lowercase().contains("name"));
            
            let has_build = obj.keys()
                .any(|k| k.to_lowercase().contains("build"));
                
            let has_language = obj.keys()
                .any(|k| k.to_lowercase().contains("language"));

            // Build error message for missing fields
            let mut missing = Vec::new();
            if !has_name { missing.push("name"); }
            if !has_language { missing.push("language"); }
            if !has_build { missing.push("build configuration"); }

            if !missing.is_empty() {
                return Err(anyhow!("Generated JSON is missing required fields ({}). Found JSON: {}", 
                    missing.join(", "), json_str));
            }

            // Create normalized version with standard field names
            let mut new_config = obj.clone();
            
            // Normalize name field
            if !obj.contains_key("name") {
                if let Some(name_key) = obj.keys()
                    .find(|k| k.to_lowercase().contains("name")) {
                    if let Some(name_value) = obj.get(name_key) {
                        new_config.remove(name_key);
                        new_config.insert("name".to_string(), name_value.clone());
                    }
                }
            }

            // Normalize language field
            if !obj.contains_key("language") {
                if let Some(lang_key) = obj.keys()
                    .find(|k| k.to_lowercase().contains("language")) {
                    if let Some(lang_value) = obj.get(lang_key) {
                        new_config.remove(lang_key);
                        new_config.insert("language".to_string(), lang_value.clone());
                    }
                }
            }

            // Normalize build field - use build_config consistently
            if !obj.contains_key("build_config") {
                if let Some(build_key) = obj.keys()
                    .find(|k| k.to_lowercase().contains("build")) {
                    if let Some(build_value) = obj.get(build_key) {
                        new_config.remove(build_key);
                        new_config.insert("build_config".to_string(), build_value.clone());
                    }
                }
            }

            Ok(serde_json::to_string(&new_config)?)
        } else {
            Err(anyhow!("Could not find valid JSON in model response"))
        }
    }

    pub async fn generate_project(&self, prompt: &str) -> Result<PathBuf> {
        // Generate project configuration
        let config_json = self.generate_project_config(prompt).await?;
        
        // Initialize state and build managers
        let state_manager = StateManager::new();
        let build_manager = BuildManager::new(state_manager, PathBuf::from("build"));
        
        // Generate the project
        let project_dir = build_manager.scaffold_project(&config_json)
            .context("Failed to generate project")?;

        Ok(project_dir)
    }

    pub async fn conditional_check(
        &self,
        _initial_prompt: &str,
        condition: &str,
        true_path: &str,
        false_path: &str,
    ) -> Result<String> {
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": Role::System,
                    "content": "You are a helpful assistant that evaluates conditions and provides responses."
                },
                {
                    "role": Role::User,
                    "content": format!(
                        "Evaluate this condition: {}\nIf true, respond with: {}\nIf false, respond with: {}",
                        condition, true_path, false_path
                    )
                }
            ],
            "temperature": 0.7
        });

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        response.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Failed to extract content from OpenAI response"))
    }

    pub async fn iterative_prompt(
        &self,
        initial_prompt: &str,
        max_iterations: usize,
        refinement_prompt: &str,
    ) -> Result<String> {
        let mut current_response = initial_prompt.to_string();

        for _ in 0..max_iterations {
            let request_body = json!({
                "model": self.model,
                "messages": [
                    {
                        "role": Role::System,
                        "content": "You are a helpful assistant that refines responses."
                    },
                    {
                        "role": Role::User,
                        "content": format!("{}\nCurrent response: {}", refinement_prompt, current_response)
                    }
                ],
                "temperature": 0.7
            });

            let client = reqwest::Client::new();
            let response = client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request_body)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let refined_response = response.get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Failed to extract content from OpenAI response"))?;

            if refined_response == current_response {
                break;
            }

            current_response = refined_response;
        }

        Ok(current_response)
    }

    /// Extract JSON from content that might contain markdown or other text
    fn extract_json_from_content(content: &str) -> Option<&str> {
        println!("Trying to extract JSON from content:\n{}", content);

        // First try to parse the entire content as JSON
        if let Ok(_) = serde_json::from_str::<serde_json::Value>(content) {
            println!("Found JSON in entire content");
            return Some(content);
        }

        // Look for JSON between code blocks (including optional language specifier)
        for block in content.split("```").skip(1).step_by(2) {
            println!("Found code block:\n{}", block);
            // Remove any language specifier if present
            let clean_json = if block.starts_with("json") {
                println!("Found json language specifier");
                block[4..].trim()
            } else {
                block.trim()
            };
            println!("Cleaned JSON:\n{}", clean_json);
            if let Ok(_) = serde_json::from_str::<serde_json::Value>(clean_json) {
                println!("Successfully parsed JSON from code block");
                return Some(clean_json);
            } else {
                println!("Failed to parse JSON from code block");
            }
        }

        // Look for JSON between curly braces
        if let Some(start) = content.find('{') {
            let mut brace_count = 0;
            let mut in_string = false;
            let mut escape_next = false;
            
            for (i, c) in content[start..].char_indices() {
                match c {
                    '{' if !in_string => brace_count += 1,
                    '}' if !in_string => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            let json_str = &content[start..=start + i];
                            println!("Found potential JSON between braces:\n{}", json_str);
                            if let Ok(_) = serde_json::from_str::<serde_json::Value>(json_str) {
                                println!("Successfully parsed JSON between braces");
                                return Some(json_str);
                            } else {
                                println!("Failed to parse JSON between braces");
                            }
                        }
                    },
                    '"' if !escape_next => in_string = !in_string,
                    '\\' if in_string => escape_next = true,
                    _ => escape_next = false,
                }
            }
        }

        println!("Could not find any valid JSON");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_project() -> Result<()> {
        // Skip this test if no API key is set
        match std::env::var("INFERENCE_API_KEY") {
            Ok(_) => (),
            Err(_) => {
                println!("Skipping test_generate_project: No INFERENCE_API_KEY set");
                return Ok(());
            }
        }

        let client = InferenceClient::new()?;
        let prompt = "Create a simple Rust web server project";
        let project_dir = client.generate_project(prompt).await?;

        assert!(project_dir.exists());
        assert!(project_dir.join("Cargo.toml").exists());
        assert!(project_dir.join("src").exists());
        assert!(project_dir.join("src/main.rs").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_iterative_prompt() -> Result<()> {
        let client = InferenceClient::new()?;
        
        let initial_prompt = "Create a project configuration for a small web application";
        let refinement_instruction = "Refine the project configuration to be more scalable and include more detailed dependency management";
        
        let final_config = client.iterative_prompt(
            initial_prompt, 
            2,  // Number of iterations
            refinement_instruction
        ).await?;

        // Validate that the final config is a valid JSON
        let config_json: serde_json::Value = serde_json::from_str(&final_config)
            .expect("Final config should be a valid JSON");
        
        assert!(config_json.is_object(), "Final config should be a JSON object");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_conditional_check() -> Result<()> {
        let client = InferenceClient::new()?;
        
        let initial_prompt = "Create a project configuration for a data science project";
        let condition_prompt = "Check if the project configuration includes machine learning libraries and data processing tools";
        let option_a_prompt = "Enhance the project configuration with advanced machine learning and data science tools";
        let option_b_prompt = "Add basic data processing and visualization libraries";
        
        let final_config = client.conditional_check(
            initial_prompt, 
            condition_prompt, 
            option_a_prompt, 
            option_b_prompt
        ).await?;

        // Validate that the final config is a valid JSON
        let config_json: serde_json::Value = serde_json::from_str(&final_config)
            .expect("Final config should be a valid JSON");
        
        assert!(config_json.is_object(), "Final config should be a JSON object");
        
        Ok(())
    }
}

// Fallback mock implementation for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::*;

    #[automock]
    impl InferenceClient {
    }
}
