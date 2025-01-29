use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use anyhow::Result;

use crate::state::types::{TaskId, TaskStatus};
use crate::prompt::Prompt;

#[async_trait]
pub trait HttpClientTrait {
    async fn post(&self, url: &str, request: &ChatCompletionRequest, api_key: &str) -> Result<ChatCompletionResponse>;
}

pub struct DefaultHttpClient {
    pub client: Client,
}

#[async_trait]
impl HttpClientTrait for DefaultHttpClient {
    async fn post(&self, url: &str, request: &ChatCompletionRequest, api_key: &str) -> Result<ChatCompletionResponse> {
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(request)
            .send()
            .await?
            .json::<ChatCompletionResponse>()
            .await?;
        Ok(response)
    }
}

impl DefaultHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferenceClient<H: HttpClientTrait = DefaultHttpClient> {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub http_client: H,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCompletionChoice {
    pub message: ChatMessage,
}

impl<H: HttpClientTrait> InferenceClient<H> {
    pub fn new_with_client(
        base_url: String, 
        api_key: String, 
        model: String, 
        temperature: f32, 
        http_client: H
    ) -> Self {
        Self {
            base_url,
            api_key,
            model,
            temperature,
            http_client,
        }
    }
}

impl InferenceClient<DefaultHttpClient> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_url: env::var("INFERENCE_API_URL")?,
            api_key: env::var("INFERENCE_API_KEY")?,
            model: env::var("INFERENCE_API_MODEL")?,
            temperature: env::var("INFERENCE_API_TEMPERATURE")?
                .parse()
                .unwrap_or(0.1),
            http_client: DefaultHttpClient::new(),
        })
    }
}

impl<H: HttpClientTrait> InferenceClient<H> {
    pub async fn execute_task_prompt(
        &self, 
        prompt: &Prompt, 
        _task_id: &TaskId
    ) -> Result<(String, TaskStatus), Box<dyn std::error::Error>> {
        // Construct chat messages from the prompt
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: prompt.system_context.clone(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.user_request.clone(),
            }
        ];

        // Optionally include build context if available
        if let Some(context) = &prompt.build_context {
            messages.push(ChatMessage {
                role: "context".to_string(),
                content: context.clone(),
            });
        }

        // Prepare the request payload
        let request_payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
        };

        // Send request to inference API
        let response = self.http_client
            .post(&format!("{}/chat/completions", self.base_url), &request_payload, &self.api_key)
            .await?;

        // Extract the first choice's message content
        let response_content = response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        // Determine task status based on response
        // This is a simple heuristic and might need refinement
        let task_status = if !response_content.is_empty() {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };

        Ok((response_content, task_status))
    }
}

// Example usage in a task execution context
pub async fn process_task<H: HttpClientTrait>(
    inference_client: &InferenceClient<H>, 
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
