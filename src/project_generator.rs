use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDesign {
    pub project_name: String,
    pub system_architecture: Option<String>,
    pub design_principles: Option<Vec<String>>,
    pub component_responsibilities: Option<HashMap<String, String>>,
    pub error_handling: Option<ErrorHandling>,
    pub performance_scalability: Option<PerformanceScalability>,
    pub logging_monitoring: Option<LoggingMonitoring>,
    pub configuration_management: Option<ConfigurationManagement>,
    pub recommendations: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorHandling {
    pub timeout: Option<u32>,
    pub retry: Option<u32>,
    pub error_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceScalability {
    pub caching: Option<String>,
    pub connection_pooling: Option<String>,
    pub load_balancing: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingMonitoring {
    pub log_levels: Option<Vec<String>>,
    pub log_storage: Option<String>,
    pub monitoring_tools: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationManagement {
    pub config_file_format: Option<String>,
    pub environment_variables: Option<Vec<String>>,
    pub config_loading: Option<String>,
}

impl ProjectDesign {
    pub fn generate_project_structure(&self) -> Result<(), std::io::Error> {
        // Create project root directory
        let project_root = Path::new(&self.project_name);
        fs::create_dir_all(&project_root)?;

        // Create standard directories
        let dirs = vec![
            "src",
            "tests",
            "docs",
            "examples",
            ".github/workflows"
        ];
        for dir in dirs {
            fs::create_dir_all(project_root.join(dir))?;
        }

        // Generate key files
        self.generate_readme(project_root)?;
        self.generate_requirements(project_root)?;
        self.generate_main_script(project_root)?;
        self.generate_config_file(project_root)?;
        self.generate_error_handler(project_root)?;
        self.generate_logging_config(project_root)?;

        Ok(())
    }

    fn generate_readme(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut readme = File::create(project_root.join("README.md"))?;
        writeln!(readme, "# {}", self.project_name)?;
        
        if let Some(arch) = &self.system_architecture {
            writeln!(readme, "\n## System Architecture\n{}", arch)?;
        }

        if let Some(principles) = &self.design_principles {
            writeln!(readme, "\n## Design Principles")?;
            for principle in principles {
                writeln!(readme, "- {}", principle)?;
            }
        }

        Ok(())
    }

    fn generate_requirements(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut req_file = File::create(project_root.join("requirements.txt"))?;
        writeln!(req_file, "openai>=1.0.0")?;
        writeln!(req_file, "python-dotenv>=0.21.0")?;
        
        // Add logging and monitoring dependencies
        if let Some(logging) = &self.logging_monitoring {
            if logging.monitoring_tools.is_some() {
                writeln!(req_file, "prometheus-client>=0.16.0")?;
            }
        }

        Ok(())
    }

    fn generate_main_script(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut main_script = File::create(project_root.join("src/main.py"))?;
        writeln!(main_script, r#"
import openai
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class OpenAIClient:
    def __init__(self):
        self.api_key = os.getenv('OPENAI_API_KEY')
        if not self.api_key:
            raise ValueError("No OpenAI API key found. Please set OPENAI_API_KEY.")
        
        openai.api_key = self.api_key

    def stream_completion(self, prompt, model="gpt-3.5-turbo"):
        try:
            response = openai.ChatCompletion.create(
                model=model,
                messages=[{{"role": "user", "content": prompt}}],
                stream=True
            )
            
            for chunk in response:
                if chunk.choices[0].delta.get('content'):
                    yield chunk.choices[0].delta.content
        except Exception as e:
            print("Error in stream completion: {{e}}")
            yield None

def main():
    client = OpenAIClient()
    prompt = "Explain quantum computing in simple terms"
    
    print("Streaming response:")
    for chunk in client.stream_completion(prompt):
        if chunk:
            print(chunk, end='', flush=True)

if __name__ == "__main__":
    main()
"#)?;

        Ok(())
    }

    fn generate_config_file(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut config_file = File::create(project_root.join(".env.example"))?;
        writeln!(config_file, "OPENAI_API_KEY=your_openai_api_key_here")?;
        
        if let Some(config_mgmt) = &self.configuration_management {
            if let Some(env_vars) = &config_mgmt.environment_variables {
                for var in env_vars {
                    writeln!(config_file, "{}=", var)?;
                }
            }
        }

        Ok(())
    }

    fn generate_error_handler(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut error_handler = File::create(project_root.join("src/error_handler.py"))?;
        writeln!(error_handler, r#"
import time
import logging

class RetryableError(Exception):
    """Custom exception for errors that can be retried."""
    pass

def retry_with_backoff(
    func, 
    max_retries=3, 
    base_delay=1, 
    max_delay=60
):
    """
    Retry a function with exponential backoff.
    
    :param func: Function to retry
    :param max_retries: Maximum number of retry attempts
    :param base_delay: Initial delay between retries
    :param max_delay: Maximum delay between retries
    :return: Result of the function
    """
    retries = 0
    delay = base_delay

    while retries < max_retries:
        try:
            return func()
        except RetryableError as error:
            logging.warning("Attempt {{retries + 1}} failed: {{error}}")
            time.sleep(delay)
            delay = min(delay * 2, max_delay)
            retries += 1

    raise Exception("Function failed after {{max_retries}} attempts")
"#)?;

        Ok(())
    }

    fn generate_logging_config(&self, project_root: &Path) -> Result<(), std::io::Error> {
        let mut logging_config = File::create(project_root.join("src/logging_config.py"))?;
        writeln!(logging_config, r#"
import logging
import sys

def setup_logging(log_level='INFO', log_file=None):
    """
    Configure logging for the application.
    
    :param log_level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
    :param log_file: Optional file to log to. If None, logs to console.
    """
    # Convert log level string to logging constant
    log_level = getattr(logging, log_level.upper())

    # Configure basic logging
    logging_config = {{
        "level": log_level,
        "format": "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        "handlers": []
    }}

    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(log_level)
    logging_config["handlers"].append(console_handler)

    # File handler if log_file is provided
    if log_file:
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(log_level)
        logging_config["handlers"].append(file_handler)

    logging.basicConfig(**logging_config)

    return logging.getLogger(__name__)
"#)?;

        Ok(())
    }
}

use crate::tools::{ExecutableTool, Tool, ToolParameters, ParameterDefinition};

impl ExecutableTool for ProjectDesign {
    fn execute(&self, arguments: &str) -> Result<String, String> {
        // Parse arguments
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| format!("Invalid arguments: {}", e))?;

        // Extract project name and other parameters
        let project_name = args.get("project_name")
            .and_then(|v| v.as_str())
            .unwrap_or("default_project");

        // Generate project
        match self.generate_project_structure() {
            Ok(_) => Ok(format!("Project '{}' generated successfully", project_name)),
            Err(e) => Err(format!("Project generation failed: {}", e)),
        }
    }
}

impl ProjectDesign {
    pub fn get_tool_definition() -> Tool {
        Tool {
            name: "generate_project".to_string(),
            description: "Generate a software project based on a design specification".to_string(),
            parameters: ToolParameters {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("project_name".to_string(), ParameterDefinition {
                        param_type: "string".to_string(),
                        description: Some("Name of the project to generate".to_string()),
                        enum_values: None,
                    }),
                    ("language".to_string(), ParameterDefinition {
                        param_type: "string".to_string(),
                        description: Some("Primary programming language".to_string()),
                        enum_values: Some(vec![
                            "python".to_string(), 
                            "rust".to_string(), 
                            "javascript".to_string()
                        ]),
                    }),
                    ("project_type".to_string(), ParameterDefinition {
                        param_type: "string".to_string(),
                        description: Some("Type of project to generate".to_string()),
                        enum_values: Some(vec![
                            "api_client".to_string(),
                            "web_app".to_string(),
                            "cli_tool".to_string(),
                            "library".to_string()
                        ]),
                    })
                ]),
                required: vec!["project_name".to_string()],
            },
        }
    }
}

pub fn parse_project_design(design_json: &str) -> Result<ProjectDesign, serde_json::Error> {
    serde_json::from_str(design_json)
}

pub fn generate_project(design_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_design = parse_project_design(design_json)?;
    project_design.generate_project_structure()?;
    Ok(())
}
