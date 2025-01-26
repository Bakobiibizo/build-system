use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct BuildCommand {
    #[arg(short, long)]
    pub target: String,
    
    #[arg(short, long)]
    pub config: Option<String>,
}

pub struct CliManager {
    _config_path: Option<String>,
}

impl CliManager {
    pub fn new(config_path: Option<String>) -> Self {
        Self {
            _config_path: config_path,
        }
    }

    pub async fn run(&self) -> Result<()> {
        Ok(())
    }

    pub async fn process_command(&self, _command: BuildCommand) -> Result<()> {
        Ok(())
    }
}
