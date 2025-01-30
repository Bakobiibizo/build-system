use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        CreateChatCompletionRequest, 
        ChatCompletionRequestMessage,
        Role,
    }
};
use anyhow::Result;

/// Language Model interaction utilities
pub struct LanguageModelClient {
    client: Client<OpenAIConfig>,
}

impl LanguageModelClient {
    /// Create a new LLM client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Generate a response using the chat completion API
    pub async fn generate_text(
        &self, 
        messages: Vec<ChatCompletionRequestMessage>, 
        model: &str
    ) -> Result<String> {
        let request = CreateChatCompletionRequest {
            model: model.to_string(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(500),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        
        // Extract the first choice's message content
        let content = response.choices.first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }
}

/// Utility function to create system and user messages
pub fn create_messages(
    system_prompt: &str, 
    user_prompt: &str
) -> Vec<ChatCompletionRequestMessage> {
    vec![
        ChatCompletionRequestMessage::System(
            async_openai::types::ChatCompletionRequestSystemMessage {
                role: Role::System,
                content: system_prompt.to_string(),
                name: None,
            }
        ),
        ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                role: Role::User,
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    user_prompt.to_string()
                ),
                name: None,
            }
        )
    ]
}
