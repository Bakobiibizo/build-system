pub mod state;
pub mod prompt;
pub mod build;
pub mod doc;
pub mod cli;
pub mod integration;

#[cfg(test)]
mod test_utils {
    use anyhow::Result;
    use std::path::PathBuf;
    use std::fs;

    /// Create a temporary test directory
    pub fn create_test_dir(name: &str) -> Result<PathBuf> {
        let path = PathBuf::from(format!("test_{}", name));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Clean up a test directory
    pub fn cleanup_test_dir(path: PathBuf) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    /// Create a test file with content
    pub fn create_test_file(path: PathBuf, content: &str) -> Result<()> {
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_utils_functionality() -> Result<()> {
        use test_utils::*;

        // Test directory creation and cleanup
        let test_dir = create_test_dir("utils")?;
        assert!(test_dir.exists());

        // Test file creation
        let test_file = test_dir.join("test.txt");
        create_test_file(test_file.clone(), "test content")?;
        assert!(test_file.exists());

        // Test cleanup
        cleanup_test_dir(test_dir.clone())?;
        assert!(!test_dir.exists());

        Ok(())
    }

    #[test]
    fn test_build() {
        // Add test for build module here
    }

    #[test]
    fn test_doc() {
        // Add test for doc module here
    }
}
