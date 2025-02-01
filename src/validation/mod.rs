use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub content: String,
    pub size_bytes: u64,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildValidation {
    pub model_response: String,
    pub build_path: PathBuf,
    pub files: HashMap<String, FileContent>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BuildValidation {
    pub fn new(model_response: String, build_path: PathBuf) -> Self {
        Self {
            model_response,
            build_path,
            files: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn add_file(&mut self, path: String, content: String, size_bytes: u64, is_directory: bool) {
        self.files.insert(
            path,
            FileContent {
                content,
                size_bytes,
                is_directory,
            },
        );
    }

    pub fn save(&self, storage: &crate::prompt::storage::Storage) -> Result<()> {
        let key = format!(
            "build_validation_{}",
            self.timestamp.format("%Y%m%d_%H%M%S")
        );
        storage.store(&key, self)?;
        Ok(())
    }

    pub fn load(
        storage: &crate::prompt::storage::Storage,
        key: &str,
    ) -> Result<Option<BuildValidation>> {
        storage.load(key)
    }
}

pub fn capture_build_output(
    build_path: PathBuf,
    model_response: String,
) -> Result<BuildValidation> {
    use std::fs;

    let mut validation = BuildValidation::new(model_response, build_path.clone());

    fn visit_dirs(dir: &PathBuf, validation: &mut BuildValidation, base_path: &PathBuf) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let relative_path = path.strip_prefix(base_path)?.to_string_lossy().into_owned();

                if path.is_dir() {
                    validation.add_file(
                        relative_path,
                        String::new(),
                        0,
                        true,
                    );
                    visit_dirs(&path, validation, base_path)?;
                } else {
                    let content = fs::read_to_string(&path)?;
                    let metadata = fs::metadata(&path)?;
                    validation.add_file(
                        relative_path,
                        content,
                        metadata.len(),
                        false,
                    );
                }
            }
        }
        Ok(())
    }

    visit_dirs(&build_path, &mut validation, &build_path)?;
    Ok(validation)
}

pub fn validate_build(validation: &BuildValidation) -> Result<ValidationReport> {
    // TODO: Implement validation logic to compare model response with actual files
    Ok(ValidationReport {
        timestamp: validation.timestamp,
        build_path: validation.build_path.clone(),
        matches: vec![],
        mismatches: vec![],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationMatch {
    pub file_path: String,
    pub expected: String,
    pub actual: String,
    pub match_type: MatchType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Partial,
    Missing,
    Unexpected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub build_path: PathBuf,
    pub matches: Vec<ValidationMatch>,
    pub mismatches: Vec<ValidationMatch>,
}
