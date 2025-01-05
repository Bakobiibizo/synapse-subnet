use crate::module::{Module, ModuleType};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use std::sync::{Arc, RwLock};
use std::path;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Module already exists: {0}")]
    ModuleExists(String),
    #[error("Invalid module configuration: {0}")]
    InvalidConfig(String),
    #[error("Module operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub module_type: ModuleType,
    pub status: ModuleStatus,
}

impl ModuleConfig {
    pub fn new(name: String, module_type: ModuleType) -> Self {
        Self {
            name,
            module_type,
            status: ModuleStatus::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatus {
    pub state: ModuleState,
    pub last_error: Option<String>,
    pub uptime: Option<u64>,
}

impl ModuleStatus {
    pub fn new() -> Self {
        Self {
            state: ModuleState::Stopped,
            last_error: None,
            uptime: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModuleState {
    Running,
    Stopped,
    Error,
}

#[async_trait]
pub trait Registry: Send + Sync {
    /// List all registered modules
    async fn list_modules(&self) -> Result<Vec<ModuleConfig>, RegistryError>;

    /// Get a module by name
    async fn get_module(&self, name: &str) -> Result<ModuleConfig, RegistryError>;

    /// Register a new module
    async fn register_module(&self, config: ModuleConfig) -> Result<(), RegistryError>;

    /// Update an existing module
    async fn update_module(&self, name: &str, config: ModuleConfig) -> Result<(), RegistryError>;

    /// Remove a module
    async fn remove_module(&self, name: &str) -> Result<(), RegistryError>;

    /// Get the status of a module
    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, RegistryError>;
}

/// A local registry for managing inference modules
pub struct LocalRegistry {
    storage_path: PathBuf,
    modules: Arc<RwLock<HashMap<String, ModuleConfig>>>,
}

impl LocalRegistry {
    /// Create a new registry with the given storage path
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the absolute path for a module
    fn get_module_path(&self, module_path: &str) -> PathBuf {
        let path = PathBuf::from(module_path);
        if path.is_absolute() {
            path
        } else {
            self.storage_path.join(module_path)
        }
    }
}

#[async_trait]
impl Registry for LocalRegistry {
    async fn list_modules(&self) -> Result<Vec<ModuleConfig>, RegistryError> {
        Ok(self.modules.read().unwrap().values().cloned().collect())
    }

    async fn get_module(&self, name: &str) -> Result<ModuleConfig, RegistryError> {
        self.modules.read().unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| RegistryError::ModuleNotFound(name.to_string()))
    }

    async fn register_module(&self, config: ModuleConfig) -> Result<(), RegistryError> {
        let mut modules = self.modules.write().unwrap();
        if modules.contains_key(&config.name) {
            return Err(RegistryError::ModuleExists(config.name));
        }
        modules.insert(config.name.clone(), config);
        Ok(())
    }

    async fn update_module(&self, name: &str, config: ModuleConfig) -> Result<(), RegistryError> {
        let mut modules = self.modules.write().unwrap();
        if !modules.contains_key(name) {
            return Err(RegistryError::ModuleNotFound(name.to_string()));
        }
        modules.insert(name.to_string(), config);
        Ok(())
    }

    async fn remove_module(&self, name: &str) -> Result<(), RegistryError> {
        let mut modules = self.modules.write().unwrap();
        if modules.remove(name).is_none() {
            return Err(RegistryError::ModuleNotFound(name.to_string()));
        }
        Ok(())
    }

    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, RegistryError> {
        let config = self.get_module(name).await?;
        Ok(config.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn create_test_config(name: &str) -> ModuleConfig {
        ModuleConfig {
            name: name.to_string(),
            module_type: ModuleType::Python {
                module_path: PathBuf::from("test.py"),
                requirements_path: None,
                venv_path: None,
            },
            status: ModuleStatus::new(),
        }
    }

    fn create_test_registry() -> LocalRegistry {
        LocalRegistry::new(PathBuf::from("/tmp/test_registry"))
    }

    #[test]
    fn test_register_module() {
        let mut registry = create_test_registry();
        let config = create_test_config("test");
        assert!(registry.register_module(config.clone()).is_ok());
        assert!(registry.register_module(config).is_err());
    }

    #[test]
    fn test_list_modules() {
        let mut registry = create_test_registry();
        let config1 = create_test_config("test1");
        let config2 = create_test_config("test2");
        registry.register_module(config1).unwrap();
        registry.register_module(config2).unwrap();
        assert_eq!(registry.list_modules().unwrap().len(), 2);
    }

    #[test]
    fn test_update_module() {
        let mut registry = create_test_registry();
        let config = create_test_config("test");
        registry.register_module(config.clone()).unwrap();
        let mut updated_config = config.clone();
        updated_config.status.state = ModuleState::Running;
        assert!(registry.update_module(&config.name, updated_config).is_ok());
        assert!(registry
            .update_module("nonexistent", config)
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[test]
    fn test_remove_module() {
        let mut registry = create_test_registry();
        let config = create_test_config("test");
        registry.register_module(config).unwrap();
        assert!(registry.remove_module("test").is_ok());
        assert!(registry
            .remove_module("test")
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[test]
    fn test_get_module() {
        let mut registry = create_test_registry();
        let config = create_test_config("test");
        registry.register_module(config.clone()).unwrap();
        assert_eq!(registry.get_module("test").unwrap().name, config.name);
        assert!(registry
            .get_module("nonexistent")
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[test]
    fn test_concurrent_access() {
        let registry = Arc::new(create_test_registry());
        let mut handles = vec![];

        // Spawn multiple threads to register modules
        for i in 0..5 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let config = create_test_config(&format!("test{}", i));
                registry.register_module(config).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(registry.list_modules().unwrap().len(), 5);
    }

    #[test]
    fn test_path_handling() {
        let registry = create_test_registry();
        
        // Test absolute paths
        let absolute_path = "/absolute/path/to/module.py";
        let config = ModuleConfig {
            name: "absolute_path_test".to_string(),
            module_type: ModuleType::Python {
                module_path: PathBuf::from(absolute_path),
                requirements_path: None,
                venv_path: None,
            },
            status: ModuleStatus::new(),
        };
        registry.register_module(config).unwrap();

        // Test relative paths
        let relative_path = "relative/path/to/module.py";
        let config = ModuleConfig {
            name: "relative_path_test".to_string(),
            module_type: ModuleType::Python {
                module_path: PathBuf::from(relative_path),
                requirements_path: Some(PathBuf::from("requirements.txt")),
                venv_path: Some(PathBuf::from(".venv")),
            },
            status: ModuleStatus::new(),
        };
        registry.register_module(config).unwrap();

        // Verify paths are properly resolved
        let module = registry.get_module("relative_path_test").unwrap();
        if let ModuleType::Python { module_path, requirements_path, venv_path } = &module.module_type {
            assert!(module_path.is_absolute());
            assert!(requirements_path.as_ref().unwrap().is_absolute());
            assert!(venv_path.as_ref().unwrap().is_absolute());
        }
    }

    #[test]
    fn test_error_cases() {
        let mut registry = create_test_registry();

        // Test registering module with empty name
        let config = ModuleConfig {
            name: "".to_string(),
            module_type: ModuleType::Python {
                module_path: PathBuf::from("test.py"),
                requirements_path: None,
                venv_path: None,
            },
            status: ModuleStatus::new(),
        };
        assert!(registry.register_module(config).is_err());

        // Test updating non-existent module
        let config = create_test_config("nonexistent");
        assert!(registry.update_module("nonexistent", config).is_err());

        // Test getting non-existent module
        assert!(registry.get_module("nonexistent").is_err());

        // Test removing non-existent module
        assert!(registry.remove_module("nonexistent").is_err());
    }

    #[test]
    fn test_module_status_transitions() {
        let mut registry = create_test_registry();
        let mut config = create_test_config("test_status");
        
        // Test initial status
        registry.register_module(config.clone()).unwrap();
        assert!(matches!(
            registry.get_module("test_status").unwrap().status.state,
            ModuleState::Stopped
        ));

        // Test transition to Running
        config.status.state = ModuleState::Running;
        registry.update_module("test_status", config.clone()).unwrap();
        assert!(matches!(
            registry.get_module("test_status").unwrap().status.state,
            ModuleState::Running
        ));

        // Test transition to Error
        config.status.state = ModuleState::Error;
        registry.update_module("test_status", config).unwrap();
        assert!(matches!(
            registry.get_module("test_status").unwrap().status.state,
            ModuleState::Error
        ));
    }
}
