use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sled::Db;
use uuid::Uuid;
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::Path;

/// Manages persistent storage and validation for prompts and workflows
pub struct PromptStorage {
    db: Db,
}

impl PromptStorage {
    /// Create a new PromptStorage instance
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// Validate JSON against a given schema
    pub fn validate_json(schema: &Value, data: &Value) -> Result<()> {
        // Create a 'static reference by leaking the schema
        let schema_static = Box::leak(Box::new(schema.clone()));
        let compiled_schema = JSONSchema::compile(schema_static)?;
        
        // Validate the data against the schema and collect any validation errors
        if let Err(errors) = compiled_schema.validate(data) {
            let error_messages: Vec<String> = errors
                .map(|error| error.to_string())
                .collect();
            anyhow::bail!("JSON validation failed: {}", error_messages.join(", "));
        }
        
        Ok(())
    }

    /// Store a serializable item with a UUID
    pub fn store<T: Serialize>(&self, key: &str, item: &T) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let serialized = serde_json::to_vec(item)?;
        self.db.insert(format!("{}-{}", key, id).as_bytes(), serialized)?;
        self.db.flush()?;
        Ok(id)
    }

    /// Retrieve a serializable item by its UUID
    pub fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str, id: &Uuid) -> Result<T> {
        let full_key = format!("{}-{}", key, id);
        let item_bytes = self.db.get(full_key.as_bytes())?
            .context("Item not found")?;
        
        serde_json::from_slice(&item_bytes)
            .context("Failed to deserialize item")
    }

    /// List all items of a specific type
    pub fn list<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Vec<(Uuid, T)>> {
        let prefix = format!("{}-", key);
        let items = self.db.scan_prefix(prefix.as_bytes())
            .filter_map(|res| {
                res.ok().and_then(|(k, v)| {
                    // Extract UUID from key
                    let uuid_str = std::str::from_utf8(&k)
                        .ok()?
                        .strip_prefix(&prefix)?;
                    let uuid = Uuid::parse_str(uuid_str).ok()?;
                    
                    // Deserialize value
                    serde_json::from_slice(&v).ok()
                        .map(|item| (uuid, item))
                })
            })
            .collect();

        Ok(items)
    }

    /// Delete an item by its UUID
    pub fn delete(&self, key: &str, id: &Uuid) -> Result<()> {
        let full_key = format!("{}-{}", key, id);
        self.db.remove(full_key.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    /// Flush changes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn store<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let serialized = serde_json::to_vec(value)?;
        self.db.insert(key.as_bytes(), serialized)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn load<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if let Some(data) = self.db.get(key.as_bytes())? {
            let value = serde_json::from_slice(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for res in self.db.iter() {
            let (key, _) = res?;
            if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                keys.push(key_str);
            }
        }
        Ok(keys)
    }

    pub fn clear(&self) -> Result<()> {
        self.db.clear()?;
        self.db.flush()?;
        Ok(())
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_json_validation() -> Result<()> {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name", "age"]
        });

        // Valid data
        let valid_data = json!({
            "name": "John Doe",
            "age": 30
        });
        PromptStorage::validate_json(&schema, &valid_data)?;

        // Invalid data
        let invalid_data = json!({
            "name": 123,
            "age": -5
        });
        assert!(PromptStorage::validate_json(&schema, &invalid_data).is_err());

        Ok(())
    }

    #[test]
    fn test_prompt_storage() -> Result<()> {
        let dir = tempdir()?;
        let storage = PromptStorage::new(dir.path())?;

        // Test storing and retrieving a simple struct
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestPrompt {
            name: String,
            description: String,
        }

        let prompt = TestPrompt {
            name: "Test Prompt".to_string(),
            description: "A test prompt for storage".to_string(),
        };

        let id = storage.store("prompt", &prompt)?;
        let retrieved_prompt: TestPrompt = storage.retrieve("prompt", &id)?;

        assert_eq!(prompt, retrieved_prompt);

        // Test listing
        let prompts = storage.list::<TestPrompt>("prompt")?;
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].1, prompt);

        // Test deletion
        storage.delete("prompt", &id)?;
        let prompts = storage.list::<TestPrompt>("prompt")?;
        assert_eq!(prompts.len(), 0);

        Ok(())
    }

    #[test]
    fn test_storage_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let storage = Storage::new(temp_dir.path())?;

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            field: String,
        }

        let test_data = TestData {
            field: "test".to_string(),
        };

        // Test store and load
        storage.store("test_key", &test_data)?;
        let loaded: TestData = storage.load("test_key")?.unwrap();
        assert_eq!(loaded, test_data);

        // Test delete
        storage.delete("test_key")?;
        assert!(storage.load::<TestData>("test_key")?.is_none());

        // Test list_keys
        storage.store("key1", &test_data)?;
        storage.store("key2", &test_data)?;
        let keys = storage.list_keys()?;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));

        Ok(())
    }
}
