use std::env;
use mockall::mock;
use mockall::predicate;
use tokio;

use crate::state::types::{TaskId, TaskStatus};
use crate::inference::{
    InferenceClient, 
    HttpClientTrait, 
    ChatCompletionRequest, 
    ChatCompletionResponse,
    ChatCompletionChoice,
    ChatMessage,
};
use crate::prompt::Prompt;

// Mock HTTP Client for testing
mock! {
    HttpClient {}
    
    #[async_trait::async_trait]
    impl HttpClientTrait for HttpClient {
        async fn post(&self, url: &str, request: &ChatCompletionRequest, api_key: &str) -> anyhow::Result<ChatCompletionResponse>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to set up test environment variables
    fn setup_test_env() {
        env::set_var("INFERENCE_API_URL", "https://test-api.com");
        env::set_var("INFERENCE_API_KEY", "test-key");
        env::set_var("INFERENCE_API_MODEL", "test-model");
        env::set_var("INFERENCE_API_TEMPERATURE", "0.1");
    }

    #[tokio::test]
    async fn test_inference_client_initialization() {
        setup_test_env();

        let client = InferenceClient::new().expect("Failed to create inference client");
        
        assert_eq!(client.base_url, "https://test-api.com");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.model, "test-model");
        assert_eq!(client.temperature, 0.1);
    }

    #[tokio::test]
    async fn test_execute_task_prompt_success() {
        setup_test_env();

        let mut mock_http_client = MockHttpClient::new();

        // Setup mock expectations
        mock_http_client
            .expect_post()
            .with(
                predicate::eq("https://test-api.com/chat/completions"),
                predicate::function(|req: &ChatCompletionRequest| {
                    req.model == "test-model" && 
                    req.temperature == 0.1 && 
                    req.messages.len() == 2
                }),
                predicate::eq("test-key")
            )
            .returning(|_, _, _| {
                Ok(ChatCompletionResponse {
                    choices: vec![
                        ChatCompletionChoice {
                            message: ChatMessage {
                                role: "assistant".to_string(),
                                content: "Test response".to_string(),
                            }
                        }
                    ]
                })
            });

        let client = InferenceClient::new_with_client(
            "https://test-api.com".to_string(), 
            "test-key".to_string(), 
            "test-model".to_string(), 
            0.1, 
            mock_http_client
        );

        let prompt = Prompt {
            system_context: "You are a helpful assistant".to_string(),
            user_request: "Generate a test response".to_string(),
            build_context: None,
        };

        let task_id = TaskId("test-task".to_string());

        let result = client.execute_task_prompt(&prompt, &task_id).await;
        
        assert!(result.is_ok(), "Result should be Ok, got {:?}", result);
        let (response, status) = result.unwrap();
        
        assert_eq!(response, "Test response");
        assert_eq!(status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_execute_task_prompt_empty_response() {
        setup_test_env();

        let mut mock_http_client = MockHttpClient::new();

        // Setup mock expectations for empty response
        mock_http_client
            .expect_post()
            .with(
                predicate::eq("https://test-api.com/chat/completions"),
                predicate::function(|req: &ChatCompletionRequest| {
                    req.model == "test-model" && 
                    req.temperature == 0.1 && 
                    req.messages.len() == 2
                }),
                predicate::eq("test-key")
            )
            .returning(|_, _, _| {
                Ok(ChatCompletionResponse {
                    choices: vec![]
                })
            });

        let client = InferenceClient::new_with_client(
            "https://test-api.com".to_string(), 
            "test-key".to_string(), 
            "test-model".to_string(), 
            0.1, 
            mock_http_client
        );

        let prompt = Prompt {
            system_context: "You are a helpful assistant".to_string(),
            user_request: "Generate a test response".to_string(),
            build_context: None,
        };

        let task_id = TaskId("test-task".to_string());

        let result = client.execute_task_prompt(&prompt, &task_id).await;
        
        assert!(result.is_ok(), "Result should be Ok, got {:?}", result);
        let (response, status) = result.unwrap();
        
        assert_eq!(response, "");
        assert_eq!(status, TaskStatus::Failed);
    }

    #[tokio::test]
    #[should_panic(expected = "environment variable not found")]
    async fn test_inference_client_missing_env_var() {
        // Clear all test environment variables
        env::remove_var("INFERENCE_API_URL");
        env::remove_var("INFERENCE_API_KEY");
        env::remove_var("INFERENCE_API_MODEL");
        env::remove_var("INFERENCE_API_TEMPERATURE");

        // This should panic due to missing environment variables
        let _ = InferenceClient::new().expect("Should panic on missing env vars");
    }
}
