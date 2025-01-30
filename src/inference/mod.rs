use anyhow::{Context, Result};
use async_openai::{
    config::{OpenAIConfig},
    Client,
    types::{
        CreateChatCompletionRequestArgs,
        Role as ChatCompletionMessageRole,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestMessage,
    }
};
use regex::Regex;
use std::env;
use tracing::{info, error};

use crate::prompt::Prompt;
use crate::state::types::TaskId;

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
    client: Client<OpenAIConfig>,
    config: OpenAIConfigWrapper,
}

impl InferenceClient {
    pub fn new() -> Result<Self> {
        // Read OpenAI API key from environment variable
        let api_key = env::var("INFERENCE_API_KEY")
            .context("INFERENCE_API_KEY must be set")?;
        let api_base = env::var("INFERENCE_API_BASE_URL")
            .context("INFERENCE_API_BASE_URL must be set")?;

        info!("Initializing InferenceClient with base URL: {}", api_base);

        // Create a new configuration with explicit base URL
        let config = OpenAIConfigWrapper::new(
            OpenAIConfig::new()
                .with_api_base(api_base.clone())
                .with_api_key(api_key.clone())
        );

        let client = Client::with_config(config.inner().clone());

        Ok(Self { 
            client,
            config: config.clone() 
        })  
    }

    pub fn get_config(&self) -> &OpenAIConfigWrapper {
        &self.config
    }

    pub async fn generate_project_config(&self, prompt: &str) -> Result<String> {
        info!("Generating project config with prompt: {}", prompt);

        // Modify the prompt to explicitly request JSON
        let json_prompt = format!(
            "{}\n\nRespond ONLY with a valid JSON configuration enclosed in ```json ... ``` code blocks. Do NOT include any explanatory text.",
            prompt
        );

        // Prepare the chat completion request
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    role: ChatCompletionMessageRole::User,
                    content: ChatCompletionRequestUserMessageContent::Text(
                        json_prompt.to_string()
                    ),
                    name: None,
                }
            )
        ];

        // Read model-specific parameters from environment variables
        let model = env::var("INFERENCE_API_MODEL").unwrap_or_else(|_| "gpt-4-turbo".to_string());
        let temperature = env::var("INFERENCE_API_TEMPERATURE")
            .map(|t| t.parse().unwrap_or(0.1))
            .unwrap_or(0.1);

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .max_tokens(4096u16)
            .temperature(temperature)
            .top_p(1.0)
            .frequency_penalty(0.0)
            .presence_penalty(0.0)
            .build()
            .map_err(|e| {
                error!("Failed to build request: {:?}", e);
                e
            })?;

        // Send the request and get the response
        let response = self.client.chat().create(request)
            .await
            .map_err(|e| {
                error!("Failed to generate project config: {:?}", e);
                anyhow::anyhow!("API request failed: {}", e)
            })?;

        // Extract the content from the first choice
        let generated_content = response.choices.first()
            .and_then(|choice| choice.message.content.clone())
            .context("No response from API")?;

        // Log the generated content for debugging
        info!("Generated Project Config Content: {}", generated_content);

        // JSON extraction strategies as trait objects
        let json_extraction_strategies: &[&dyn Fn(&str) -> Option<String>] = &[
            // Strategy 1: Extract full JSON from ```json ... ``` code blocks
            &|content: &str| {
                let code_block_regex = Regex::new(r"```json\s*(\{.*?\})\s*```").unwrap();
                code_block_regex.captures(content)
                    .and_then(|caps| caps.get(1))
                    .map(|m| m.as_str().to_string())
            },
            // Strategy 2: Extract full multi-line JSON from ```json ... ``` code blocks
            &|content: &str| {
                let multi_line_json_regex = Regex::new(r"```json\s*(\{[\s\S]*?\})\s*```").unwrap();
                multi_line_json_regex.captures(content)
                    .and_then(|caps| caps.get(1))
                    .map(|m| m.as_str().to_string())
            },
            // Strategy 3: Extract first complete JSON object
            &|content: &str| {
                let json_regex = Regex::new(r"\{[\s\S]*?\}").unwrap();
                json_regex.find(content)
                    .map(|m| m.as_str().to_string())
            },
            // Strategy 4: Trim and clean the entire content
            &|content: &str| {
                let trimmed = content.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            }
        ];

        // Try each JSON extraction strategy
        for &strategy in json_extraction_strategies {
            if let Some(json_str) = strategy(&generated_content) {
                // Validate the extracted JSON
                match serde_json::from_str::<serde_json::Value>(&json_str) {
                    Ok(_) => return Ok(json_str),
                    Err(e) => {
                        error!("Invalid JSON from strategy: {}", e);
                        continue;
                    }
                }
            }
        }

        // If no valid JSON is found, return an error
        Err(anyhow::anyhow!(
            "Failed to extract valid JSON from the generated content. Content was: {}",
            generated_content
        ))
    }

    pub async fn execute_task_prompt(&self, prompt: &Prompt, task_id: &TaskId) -> Result<String> {
        info!("Executing task prompt for task ID: {}", task_id);

        // Prepare the chat completion request
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    role: ChatCompletionMessageRole::User,
                    content: ChatCompletionRequestUserMessageContent::Text(
                        format!("Task ID: {}\nSystem Context: {}\nUser Request: {}", 
                            task_id, 
                            prompt.system_context, 
                            prompt.user_request
                        )
                    ),
                    name: None,
                }
            )
        ];

        // Read model-specific parameters from environment variables
        let model = env::var("INFERENCE_API_MODEL").unwrap_or_else(|_| "gpt-4-turbo".to_string());
        let temperature = env::var("INFERENCE_API_TEMPERATURE")
            .map(|t| t.parse().unwrap_or(0.1))
            .unwrap_or(0.1);

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .max_tokens(4096u16)
            .temperature(temperature)
            .top_p(1.0)
            .frequency_penalty(0.0)
            .presence_penalty(0.0)
            .build()
            .map_err(|e| {
                error!("Failed to build request: {:?}", e);
                e
            })?;

        // Send the request to OpenAI
        let response = self.client.chat().create(request).await
            .map_err(|e| {
                error!("Failed to create chat completion: {:?}", e);
                e
            })?;

        // Extract and return the generated content
        let content = response.choices.first()
            .and_then(|choice| choice.message.content.clone())
            .context("No content in response")?;

        Ok(content)
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
