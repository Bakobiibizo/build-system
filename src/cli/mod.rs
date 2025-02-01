use crate::{
    doc::{
        types::{DocType, Documentation}, DocumentationEngine, FileDocumentationEngine
    }, 
    prompt::storage::Storage,
    validation::{self, BuildValidation},
    inference::InferenceClient,
    prompt::{Prompt, ProjectConfig, ProjectType},
    state::types::TaskId,
    project_generator::ProjectGenerator,
};
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub async fn run(cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Doc { .. } => handle_doc_command(cli).await,
            Commands::ValidateBuild { .. } => handle_validation_command(cli).await,
            Commands::Inference { .. } => handle_inference_command(cli).await,
            Commands::SaveModelOutput { .. } => handle_save_model_output_command(cli).await,
            Commands::Generate { .. } => handle_generate_command(cli).await,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Save model output for validation
    SaveModelOutput {
        /// Path to build directory
        #[arg(short, long)]
        build_path: String,

        /// Model response to validate
        #[arg(short, long)]
        model_response: String,

        /// Path to storage directory
        #[arg(short, long)]
        storage_path: String,
    },

    /// Validate a saved build
    ValidateBuild {
        /// Path to storage directory
        #[arg(short, long)]
        storage_path: String,

        /// Validation key
        #[arg(short, long)]
        validation_key: String,
    },

    /// Generate a new project
    Generate {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// Project language
        #[arg(short, long)]
        language: String,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Generate documentation
    Doc {
        /// Path to source directory
        #[arg(short, long)]
        source_path: PathBuf,

        /// Path to output directory
        #[arg(short, long)]
        output_path: PathBuf,

        /// Documentation type
        #[arg(short, long)]
        doc_type: DocType,
    },

    /// Run inference
    Inference {
        /// Task ID
        #[arg(short, long)]
        task_id: String,

        /// System context
        #[arg(short, long)]
        system_context: String,

        /// User request
        #[arg(short, long)]
        user_request: String,
    },
}

pub async fn handle_cli_command(cli: Cli) -> Result<()> {
    Cli::run(cli).await
}

pub async fn handle_doc_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Doc {
            source_path,
            output_path: _,
            doc_type,
        } => {
            let engine = FileDocumentationEngine::new(source_path.clone());
            let _doc = engine.create_doc(&Documentation::new(
                "Generated Documentation".to_string(),
                "".to_string(),
                doc_type,
                source_path,
                "build-system".to_string(),
            )).await?;
            println!("Documentation generated successfully");
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub async fn handle_validation_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::ValidateBuild {
            storage_path,
            validation_key,
        } => {
            // Initialize storage
            let storage = Storage::new(PathBuf::from(storage_path.clone()))?;

            // Load the validation data
            let validation = BuildValidation::load(&storage, &validation_key)?
                .ok_or_else(|| anyhow!("Validation data not found for key: {}", validation_key))?;

            // Run validation
            let report = validation::validate_build(&validation)?;
            println!("Validation Report:");
            println!("Timestamp: {}", report.timestamp);
            println!("Build Path: {}", report.build_path.display());
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub async fn handle_inference_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Inference {
            task_id,
            system_context,
            user_request,
        } => {
            let client = InferenceClient::new()?;
            let prompt = Prompt::new(
                &system_context,
                &user_request,
            );
            let task_id = TaskId::new(&task_id);
            let response = client.execute_task_prompt(&prompt, &task_id).await?;
            println!("Response: {}", response);
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub async fn handle_save_model_output_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::SaveModelOutput {
            build_path,
            model_response,
            storage_path,
        } => {
            // Initialize storage
            let storage = Storage::new(PathBuf::from(storage_path.clone()))?;

            // Capture the build output and model response
            let validation = validation::capture_build_output(
                PathBuf::from(build_path),
                model_response,
            )?;

            // Save the validation data
            validation.save(&storage)?;

            println!("Model output saved successfully");
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub async fn handle_generate_command(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Generate { name, language, description } => {
            let config = ProjectConfig {
                name: name.clone(),
                language: language.clone(),
                description: description.clone(),
                project_type: ProjectType::Application,
                framework: None,
                initialization_commands: vec![].into(),
                recommendations: vec![].into(),
                build_config: None,
                directory_structure: None,
                dependencies: None,
                technologies: vec![],
            };
            let generator = ProjectGenerator::new(config);
            generator.generate()?;
            println!("Project generated successfully");
            Ok(())
        }
        _ => unreachable!(),
    }
}
