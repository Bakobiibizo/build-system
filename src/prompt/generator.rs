use crate::prompt::{Prompt, ProjectConfig};

pub struct PromptGenerator;

impl PromptGenerator {
    pub fn generate_project_prompt(config: &ProjectConfig) -> Prompt {
        let system_context = "You are a helpful AI assistant that generates project configurations.";
        let user_request = format!(
            "Generate a project configuration for: {}",
            config.name
        );
        Prompt::new(system_context, &user_request)
    }
}
