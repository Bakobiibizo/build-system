use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::collections::HashMap;
use chrono::Utc;

use crate::doc::{DocumentationEngine, FileDocumentationEngine, types::{Documentation, DocType}};
use crate::prompt::PromptManager;
use crate::project_generator::ProjectGenerationWorkflow;

#[derive(Parser)]
#[command(name = "build-system")]
#[command(about = "A powerful build and project generation system")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new project
    Generate {
        /// Name of the project
        #[arg(long)]
        name: String,

        /// Description of the project
        #[arg(long)]
        description: String,
    },

    /// Create documentation for a project
    Doc {
        /// Project name
        #[arg(long)]
        project: String,

        /// Documentation type
        #[arg(long, value_enum)]
        doc_type: DocType,

        /// Title of the documentation
        #[arg(long)]
        title: String,

        /// Content of the documentation
        #[arg(long)]
        content: String,
    },
}

/// Run the CLI with async support
pub async fn run_cli(cli: Cli) -> Result<()> {
    match &cli.command {
        Some(Commands::Generate { name, description }) => {
            let prompt_manager = PromptManager::new(".reference/templates")?;
            let description = description.clone();

            let project_config = prompt_manager
                .generate_project_config(&description)
                .await?;

            let workflow = ProjectGenerationWorkflow::new(&project_config);

            let project_path = workflow.generate_project_structure()?;

            // Create project documentation
            let doc_engine = FileDocumentationEngine::try_new(&project_path).await?;

            // Generate initial documentation
            let project_doc = Documentation {
                id: name.clone(),
                project: name.clone(),
                title: format!("{} Project Configuration", name),
                description: Some(description.clone()),
                content: description.clone(),
                doc_type: DocType::ProjectOverview,
                path: project_path.join("docs").join("PROJECT_OVERVIEW.md"),
                owner: String::new(),
                priority: String::new(),
                tags: vec![],
                additional_info: HashMap::new(),
                steps: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            };

            doc_engine.create_doc(&project_doc).await?;

            println!("Project '{}' generated successfully at: {}", name, project_path.display());
            Ok(())
        },
        Some(Commands::Doc { project, doc_type, title, content }) => {
            let doc_path = PathBuf::from("build").join(project);
            let doc_engine = FileDocumentationEngine::try_new(&doc_path).await?;

            let doc = Documentation {
                id: title.clone(),
                project: project.clone(),
                title: title.clone(),
                description: Some(content.clone()),
                content: content.clone(),
                doc_type: doc_type.clone(),
                path: doc_path.join(format!("{}.md", title)),
                owner: String::new(),
                priority: String::new(),
                tags: vec![],
                additional_info: HashMap::new(),
                steps: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            };

            doc_engine.create_doc(&doc).await?;

            Ok(())
        },
        None => Ok(()),
    }
}
