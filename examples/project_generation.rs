use anyhow::Result;
use build_system::prompt::PromptManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize prompt manager
    let template_dir = "./templates";
    let prompt_manager = PromptManager::new(template_dir)?;

    // Generate project configuration
    let generated_config = prompt_manager.generate_project_config("Create a web application").await?;

    // Print the generated configuration
    println!("Generated Project Configuration:");
    println!("{:#?}", generated_config);

    Ok(())
}
