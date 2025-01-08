use crate::docker::DockerModuleRuntime;
use crate::module::{Module, ModuleRuntime, ModuleStatus};
use crate::verification::{ModuleVerifier, VerificationError};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur during registry operations
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Module with the given name was not found
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    /// Attempted to register a module with a name that already exists
    #[error("Module already exists: {0}")]
    ModuleExists(String),
    /// Module verification failed
    #[error("Module verification failed: {0}")]
    VerificationError(#[from] VerificationError),
    /// Module runtime error
    #[error("Module runtime error: {0}")]
    RuntimeError(String),
}

/// Local registry for managing modules
pub struct LocalRegistry {
    modules: Arc<RwLock<HashMap<String, Module>>>,
    runtimes: Arc<RwLock<HashMap<String, Box<dyn ModuleRuntime + Send + Sync>>>>,
}

impl LocalRegistry {
    /// Create a new LocalRegistry
    pub async fn new() -> Result<Self, RegistryError> {
        Ok(Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a new module in the registry
    pub async fn register_module(&self, module: Module) -> Result<(), RegistryError> {
        ModuleVerifier::default().verify(&module).map_err(RegistryError::VerificationError)?;

        let mut modules = self.modules.write().await;
        let mut runtimes = self.runtimes.write().await;

        if modules.contains_key(&module.name) {
            return Err(RegistryError::ModuleExists(module.name.clone()));
        }

        let runtime = Box::new(DockerModuleRuntime::new(module.clone()).await.map_err(|e| {
            RegistryError::RuntimeError(format!("Failed to create runtime: {}", e))
        })?);

        let name = module.name.clone();
        modules.insert(name.clone(), module);
        runtimes.insert(name, runtime);

        Ok(())
    }

    /// Unregister a module from the registry
    pub async fn unregister_module(&self, name: &str) -> Result<(), RegistryError> {
        if let Some(_module) = self.modules.write().await.remove(name) {
            Ok(())
        } else {
            Err(RegistryError::ModuleNotFound(name.to_string()))
        }
    }

    /// Start a module
    pub async fn start_module(&self, name: &str) -> Result<(), RegistryError> {
        let runtimes = self.runtimes.read().await;
        if let Some(runtime) = runtimes.get(name) {
            runtime.start().await.map_err(|e| {
                RegistryError::RuntimeError(format!("Failed to start module: {}", e))
            })?;
            Ok(())
        } else {
            Err(RegistryError::ModuleNotFound(name.to_string()))
        }
    }

    /// Stop a module
    pub async fn stop_module(&self, name: &str) -> Result<(), RegistryError> {
        let runtimes = self.runtimes.read().await;
        if let Some(runtime) = runtimes.get(name) {
            runtime.stop().await.map_err(|e| {
                RegistryError::RuntimeError(format!("Failed to stop module: {}", e))
            })?;
            Ok(())
        } else {
            Err(RegistryError::ModuleNotFound(name.to_string()))
        }
    }

    /// Get module status
    pub async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, RegistryError> {
        let runtimes = self.runtimes.read().await;
        if let Some(runtime) = runtimes.get(name) {
            runtime.status().await.map_err(|e| {
                RegistryError::RuntimeError(format!("Failed to get module status: {}", e))
            })
        } else {
            Err(RegistryError::ModuleNotFound(name.to_string()))
        }
    }

    /// Get a module by name
    pub async fn get_module(&self, name: &str) -> Result<Module, RegistryError> {
        let modules = self.modules.read().await;
        if let Some(module) = modules.get(name) {
            Ok(module.clone())
        } else {
            Err(RegistryError::ModuleNotFound(name.to_string()))
        }
    }

    /// List all modules in the registry
    pub async fn list_modules(&self) -> Result<Vec<Module>, RegistryError> {
        let modules = self.modules.read().await;
        Ok(modules.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::{ModuleType, ModuleStatus};
    use std::collections::HashMap;

    fn create_test_module(name: &str) -> Module {
        Module {
            name: name.to_string(),
            module_type: ModuleType::Docker {
                image: "docker.io/library/nginx".to_string(),
                tag: "latest".to_string(),
                port: 8084,  // Using a different port
                env: Some(HashMap::from([
                    ("NGINX_PORT".to_string(), "80".to_string()),
                    ("MODULE_NAME".to_string(), name.to_string()),
                    ("MODULE_PORT".to_string(), "8084".to_string()),
                ])),
                volumes: Some(HashMap::from([
                    ("/tmp/test".to_string(), "/usr/share/nginx/html".to_string()),
                ])),
                health_check: None,
            },
            status: ModuleStatus::new(),
        }
    }

    #[tokio::test]
    async fn test_register_module() {
        let registry = LocalRegistry::new()
            .await
            .unwrap();

        let module = create_test_module("test-register");
        registry.register_module(module.clone()).await.unwrap();

        let registered = registry.get_module("test-register").await.unwrap();
        assert_eq!(registered.name, "test-register");
    }

    #[tokio::test]
    async fn test_get_module() {
        let registry = LocalRegistry::new()
            .await
            .unwrap();

        let module = create_test_module("test-get");
        registry.register_module(module.clone()).await.unwrap();

        let result = registry.get_module("test-get").await.unwrap();
        assert_eq!(result.name, "test-get");

        let result = registry.get_module("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_modules() {
        let registry = LocalRegistry::new()
            .await
            .unwrap();

        let module1 = create_test_module("test-list-1");
        let module2 = create_test_module("test-list-2");

        registry.register_module(module1).await.unwrap();
        registry.register_module(module2).await.unwrap();

        let modules = registry.list_modules().await.unwrap();
        assert_eq!(modules.len(), 2);
        assert!(modules.iter().any(|m| m.name == "test-list-1"));
        assert!(modules.iter().any(|m| m.name == "test-list-2"));
    }

    #[tokio::test]
    async fn test_unregister_module() {
        let registry = LocalRegistry::new()
            .await
            .unwrap();

        let module = create_test_module("test-unregister");
        registry.register_module(module).await.unwrap();

        registry.unregister_module("test-unregister").await.unwrap();

        let result = registry.get_module("test-unregister").await;
        assert!(result.is_err());
    }
}
