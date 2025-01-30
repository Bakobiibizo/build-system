use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn, error};

use crate::state::types::{TaskId, TaskStatus};
use crate::prompt::Prompt;

#[derive(Debug, Clone)]
pub struct InferenceClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatChoice {
    delta: Option<ChatDelta>,
    message: Option<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatDelta {
    content: Option<String>,
}

fn ensure_ai_responses_dir() -> std::io::Result<()> {
    let response_dir = Path::new(".reference/ai_responses");
    fs::create_dir_all(response_dir)?;
    Ok(())
}

impl InferenceClient {
    pub fn new() -> Result<Self> {
        ensure_ai_responses_dir()?;
        let base_url = env::var("INFERENCE_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let api_key = env::var("INFERENCE_API_KEY")
            .unwrap_or_else(|_| "sk-placeholder-key".to_string());
        let model = env::var("INFERENCE_API_MODEL")
            .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(900))
            .build()?;

        Ok(Self {
            client,
            base_url,
            api_key,
            model,
        })
    }

    pub async fn create_completion(&self, prompt: &str, temperature: f32) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            stream: false,
        };

        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        let completion_response: ChatCompletionResponse = response.json().await?;
        
        // Extract content from the first choice
        let full_response = completion_response.choices
            .first()
            .and_then(|choice| choice.delta.as_ref())
            .and_then(|delta| delta.content.clone())
            .unwrap_or_default();

        Ok(full_response)
    }

    pub async fn stream_completion(&self, prompt: &str, temperature: f32) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            stream: true,
        };

        let mut full_response = String::new();
        let response_dir = Path::new(".reference/ai_responses");
        fs::create_dir_all(response_dir)?;
        let response_path = response_dir.join(format!("stream_response_{}.txt", chrono::Utc::now().timestamp()));
        let mut response_file = fs::File::create(&response_path)?;

        let mut response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        info!("Streaming response to: {}", response_path.display());

        while let Some(chunk) = response.chunk().await? {
            if let Ok(chunk_str) = String::from_utf8(chunk.to_vec()) {
                // Handle streaming chunks
                for line in chunk_str.lines() {
                    if line.starts_with("data: ") {
                        let json_str = line.trim_start_matches("data: ").trim();
                        
                        // Skip the [DONE] marker
                        if json_str == "[DONE]" {
                            break;
                        }

                        // Attempt to parse JSON, being more lenient
                        match serde_json::from_str::<serde_json::Value>(json_str) {
                            Ok(json_value) => {
                                // Extract content from the JSON chunk
                                if let Some(content) = json_value
                                    .get("choices")
                                    .and_then(|choices| choices.get(0))
                                    .and_then(|choice| choice.get("delta"))
                                    .and_then(|delta| delta.get("content"))
                                    .and_then(|content| content.as_str()) 
                                {
                                    full_response.push_str(content);
                                    
                                    // Write to file
                                    use std::io::Write;
                                    response_file.write_all(content.as_bytes())?;
                                    response_file.flush()?;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse JSON chunk: {} (raw chunk: {})", e, json_str);
                            }
                        }
                    }
                }
            }
        }

        info!("Completed streaming. Total response length: {} characters", full_response.len());
        Ok(full_response)
    }

    pub async fn execute_task_prompt(&self, prompt: &Prompt, _task_id: &TaskId) -> Result<(String, TaskStatus)> {
        let url = format!("{}/chat/completions", self.base_url);
        
        // Prepare response directory
        let response_dir = Path::new(".reference/ai_responses");
        let response_path = response_dir.join(format!("task_response_{}.txt", chrono::Utc::now().timestamp()));
        let mut response_file = fs::File::create(&response_path)?;

        info!("Executing task prompt, response will be saved to: {}", response_path.display());

        // Prepare request body with more explicit system context
        let request_body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "You are an expert software architect. \
                        Respond ONLY with a valid JSON matching the project generation schema. \
                        Context: {}", 
                        prompt.system_context
                    ),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.user_request.clone(),
                }
            ],
            temperature: 0.1, // Low temperature for more deterministic output
            stream: true,
        };

        let mut response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        let mut full_response = String::new();
        let mut json_buffer = String::new();
        let mut in_json_block = false;
        let mut brace_count = 0;

        while let Some(chunk) = response.chunk().await? {
            if let Ok(chunk_str) = String::from_utf8(chunk.to_vec()) {
                full_response.push_str(&chunk_str);
                
                // Write full response to file
                response_file.write_all(chunk_str.as_bytes())?;
                response_file.flush()?;

                // JSON extraction logic
                for ch in chunk_str.chars() {
                    match ch {
                        '{' if !in_json_block => {
                            brace_count = 1;
                            in_json_block = true;
                            json_buffer.push(ch);
                        }
                        '{' if in_json_block => {
                            brace_count += 1;
                            json_buffer.push(ch);
                        }
                        '}' if in_json_block => {
                            brace_count -= 1;
                            json_buffer.push(ch);
                            
                            if brace_count == 0 {
                                // Potential complete JSON found
                                match serde_json::from_str::<serde_json::Value>(&json_buffer) {
                                    Ok(json_value) if json_value.is_object() => {
                                        info!("Extracted valid JSON configuration");
                                        return Ok((json_buffer.clone(), TaskStatus::Completed));
                                    }
                                    Err(e) => {
                                        warn!("Invalid JSON: {}", e);
                                    }
                                    _ => {}
                                }
                                json_buffer.clear();
                                in_json_block = false;
                            }
                        }
                        _ if in_json_block => json_buffer.push(ch),
                        _ => {}
                    }
                }
            }
        }

        // Fallback: try to extract JSON from full response
        if let Some(json_str) = full_response
            .lines()
            .find(|line| line.trim().starts_with('{') && line.trim().ends_with('}')) {
            
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(_) => {
                    info!("Extracted JSON from full response");
                    return Ok((json_str.to_string(), TaskStatus::Completed));
                }
                Err(e) => {
                    error!("Failed to parse fallback JSON: {}", e);
                }
            }
        }

        error!("No valid JSON configuration found in response");
        Err(anyhow::anyhow!("Failed to extract valid JSON from response"))
    }
}

#[async_trait]
pub trait InferenceProcessor: Send + Sync {
    async fn execute_task_prompt(&self, prompt: &Prompt, _task_id: &TaskId) -> Result<(String, TaskStatus)>;
}

#[async_trait]
impl InferenceProcessor for InferenceClient {
    async fn execute_task_prompt(&self, prompt: &Prompt, _task_id: &TaskId) -> Result<(String, TaskStatus)> {
        self.execute_task_prompt(prompt, _task_id).await
    }
}

// Example usage in a task execution context
pub async fn process_task(
    inference_client: &InferenceClient, 
    prompt: &Prompt, 
    task_id: &TaskId
) -> Result<crate::state::types::TaskState, Box<dyn std::error::Error>> {
    let (response, status) = inference_client.execute_task_prompt(prompt, task_id).await?;

    // Create a TaskState with the given task_id and status
    let mut task_state = crate::state::types::TaskState::new(task_id.clone());
    task_state.status = status;
    
    // Update metadata with additional information
    task_state.metadata.name = task_id.to_string();
    task_state.metadata.description = Some(response);
    task_state.metadata.owner = "inference_client".to_string();
    task_state.metadata.tags = vec!["inference".to_string()];

    Ok(task_state)
}
