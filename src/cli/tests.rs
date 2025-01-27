use anyhow::Result;
use std::path::PathBuf;
use crate::state::StateManager;
use crate::build::BuildManager;
use crate::cli::{CliManager, BuildCommand};
use tempfile::NamedTempFile;
use clap::Parser;

// Helper function to create a temporary config file
async fn create_temp_config() -> (NamedTempFile, String) {
    let config = NamedTempFile::new().unwrap();
    let config_path = config.path().to_str().unwrap().to_string();
    (config, config_path)
}

#[tokio::test]
async fn test_cli_manager_new() -> Result<()> {
    let state_manager = StateManager::new();
    let build_manager = BuildManager::new(state_manager, PathBuf::from("/tmp"));
    let cli = CliManager::new(None, build_manager);
    assert!(cli.execute_command("build", &["--target", "echo hello"]).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_cli_with_config() -> Result<()> {
    let state_manager = StateManager::new();
    let build_manager = BuildManager::new(state_manager, PathBuf::from("/tmp"));
    let config_path = Some("config.toml".to_string());
    let cli = CliManager::new(config_path.clone(), build_manager);
    assert!(cli.execute_command("build", &["--target", "echo hello"]).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_invalid_command() -> Result<()> {
    let state_manager = StateManager::new();
    let build_manager = BuildManager::new(state_manager, PathBuf::from("/tmp"));
    let cli = CliManager::new(None, build_manager);
    assert!(cli.execute_command("invalid", &[]).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_build_command_parse() -> Result<()> {
    let args = vec!["build", "--target", "echo hello"];
    let cmd = BuildCommand::try_parse_from(args)?;
    assert_eq!(cmd.target, "echo hello");
    assert!(cmd.working_dir.is_none());
    Ok(())
}

#[tokio::test]
async fn test_build_command_with_working_dir() -> Result<()> {
    let args = vec!["build", "--target", "echo hello", "--working-dir", "/tmp/test"];
    let cmd = BuildCommand::try_parse_from(args)?;
    assert_eq!(cmd.target, "echo hello");
    assert_eq!(cmd.working_dir, Some(PathBuf::from("/tmp/test")));
    Ok(())
}
