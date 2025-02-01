use anyhow::Result;
use build_system::inference::InferenceClient;
use build_system::prompt::{ProjectConfig, ProjectType};
use build_system::state::types::TaskId;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize AI inference client
    let inference_client = InferenceClient::new()?;

    // Define project generation prompts
    let project_prompts = vec![
        "Create a comprehensive project structure for a task management web application with user authentication",
        "Design a real-time collaborative document editing platform",
        "Develop a machine learning model deployment framework with CI/CD integration"
    ];

    // Iterate through project prompts and generate configurations
    for (index, prompt_text) in project_prompts.iter().enumerate() {
        println!("\n--- Project Generation {} ---", index + 1);
        println!("Prompt: {}", prompt_text);

        // Create a sample project configuration
        let project_config = ProjectConfig {
            name: "MyProject".to_string(),
            description: Some(prompt_text.to_string()),
            technologies: vec!["rust".to_string(), "actix-web".to_string()],
            project_type: ProjectType::Application,
            language: "rust".to_string(),
            framework: Some("actix-web".to_string()),
            dependencies: None,
            build_config: None,
            directory_structure: None,
            initialization_commands: None,
            recommendations: None,
        };

        // Generate a prompt for project generation
        let prompt = build_system::prompt::generator::PromptGenerator::generate_project_prompt(&project_config);

        // Execute AI-powered project generation
        match inference_client.execute_task_prompt(&prompt, &TaskId::new("project_generation")).await {
            Ok(response) => {
                println!("Generation Status: Success");
                println!("AI-Generated Project Configuration:\n{}", response);
            }
            Err(e) => {
                eprintln!("Project generation failed: {}", e);
            }
        }

        // Add a small delay between generations
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(())
}
