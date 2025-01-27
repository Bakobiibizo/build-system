use anyhow::Result;
use std::path::PathBuf;
use clap::Parser;
use crate::build::BuildManager;
use crate::state::types::TaskId;
use chrono::Utc;

#[derive(Debug, Parser)]
pub struct BuildCommand {
    #[clap(long)]
    pub target: String,
    #[clap(long)]
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub struct CliManager {
    config_path: Option<String>,
    build_manager: BuildManager,
}

impl CliManager {
    pub fn new(config_path: Option<String>, build_manager: BuildManager) -> Self {
        Self {
            config_path,
            build_manager,
        }
    }

    pub async fn execute_command(&self, action: &str, args: &[&str]) -> Result<()> {
        match action {
            "build" => {
                let cmd = BuildCommand::try_parse_from(std::iter::once("build").chain(args.iter().map(|s| *s)))
                    .map_err(|e| anyhow::anyhow!("Failed to parse build command: {}", e))?;
                let task_id = self.create_build_task(&Command {
                    action: "build".to_string(),
                    target: cmd.target,
                }).await?;
                self.build_manager.execute_task(&task_id).await?;
            }
            _ => anyhow::bail!("Unknown command: {}", action),
        }
        Ok(())
    }

    async fn create_build_task(&self, command: &Command) -> Result<TaskId> {
        let task_id = TaskId::new(&format!("build-{}", Utc::now().timestamp()));
        let task = crate::state::types::TaskState {
            id: task_id.clone(),
            status: crate::state::types::TaskStatus::Pending,
            metadata: crate::state::types::TaskMetadata {
                name: command.target.clone(),
                description: Some("Build task".to_string()),
                owner: "system".to_string(),
                dependencies: vec![],
                estimated_duration: std::time::Duration::from_secs(300),
                priority: 1,
                tags: vec!["build".to_string()],
                additional_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("command".to_string(), command.target.clone());
                    info
                },
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.build_manager.state_manager.create_task(task).await?;
        Ok(task_id)
    }
}

#[cfg(test)]
mod tests;
