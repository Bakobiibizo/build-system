use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Prompt {
    pub system_context: String,
    pub user_request: String,
    pub build_context: Option<String>,
}

pub trait PromptProcessor: Send + Sync {
    async fn process_response(&self, _response: String) -> Result<()>;
}

pub struct PromptManager {
    template_dir: String,
}

impl PromptProcessor for PromptManager {
    async fn process_response(&self, _response: String) -> Result<()> {
        // Process the response from the LLM
        Ok(())
    }
}

impl PromptManager {
    pub fn new(template_dir: String) -> Self {
        Self {
            template_dir,
        }
    }

    pub async fn create_prompt(&self, user_request: String, build_context: Option<String>) -> Result<Prompt> {
        Ok(Prompt {
            system_context: "You are a helpful build system assistant.".to_string(),
            user_request,
            build_context,
        })
    }
}
