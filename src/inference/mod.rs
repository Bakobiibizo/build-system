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
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not found"))?;
        let base_url = std::env::var("OPENAI_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-4".to_string());

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
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": Role::System,
                    "content": "You are a helpful assistant that generates project configurations."
                },
                {
                    "role": Role::User,
                    "content": prompt
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_project() -> Result<()> {
        // Skip this test if no API key is set
        match std::env::var("OPENAI_API_KEY") {
            Ok(_) => (),
            Err(_) => {
                println!("Skipping test_generate_project: No OPENAI_API_KEY set");
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
