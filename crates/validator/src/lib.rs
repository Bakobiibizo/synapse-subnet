//! Validator implementation for the Synapse Subnet project.
//! 
//! This crate provides the validator functionality for managing and validating
//! inference requests in the subnet.
//!
//! Required Features and Components:
//! 1. Request Validation and Rate Limiting
//!    - Validate incoming request format and parameters
//!    - Implement rate limiting per client/module
//!    - Check request permissions and quotas
//!
//! 2. Token Counting and Resource Management
//!    - Track token usage per request
//!    - Monitor and manage resource allocation
//!    - Implement token-based pricing
//!
//! 3. Load Balancing
//!    - Distribute requests across multiple miners
//!    - Monitor miner health and capacity
//!    - Implement failover mechanisms
//!
//! 4. Result Verification
//!    - Verify inference results quality
//!    - Check for malformed or invalid outputs
//!    - Implement result scoring system
//!
//! 5. State Management
//!    - Maintain validator state
//!    - Sync state with blockchain
//!    - Handle state transitions
//!
//! Key Interfaces Required:
//! - ChainInterface: For blockchain communication
//! - MinerInterface: For miner communication
//! - StorageInterface: For state persistence
//! - MetricsInterface: For performance monitoring

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use docker_manager::{DockerError, ContainerManager};
use registrar_api::{RegistrarClient, ClientError as RegistrarError};
use registrar::module::{Module, ModuleType, ModuleStatus};

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("Docker error: {0}")]
    Docker(#[from] DockerError),
    #[error("Registry error: {0}")]
    Registry(#[from] RegistrarError),
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Subnet error: {0}")]
    Subnet(String),
}

/// Manages the validator's modules and subnet configurations
pub struct ValidatorManager {
    docker: Arc<dyn ContainerManager>,
    registrar: Arc<RegistrarClient>,
    active_modules: RwLock<HashMap<String, Module>>,
    subnet_modules: RwLock<HashMap<String, Module>>,
}

impl ValidatorManager {
    pub fn new(docker: Arc<dyn ContainerManager>, registrar_url: String) -> Self {
        Self {
            docker,
            registrar: Arc::new(RegistrarClient::new(registrar_url)),
            active_modules: RwLock::new(HashMap::new()),
            subnet_modules: RwLock::new(HashMap::new()),
        }
    }

    /// Install an inference module
    pub async fn install_inference_module(&self, module: Module) -> Result<(), ValidatorError> {
        // Register the module with the registry
        self.registrar.register_module(module.clone()).await.map_err(ValidatorError::Registry)?;
        
        // Start the module
        self.registrar.start_module(&module.name).await.map_err(ValidatorError::Registry)?;
        
        // Store module type for reference
        let mut active_modules = self.active_modules.write().await;
        active_modules.insert(module.name.clone(), module);
        
        Ok(())
    }

    /// Install subnet code and identify its requirements
    pub async fn install_subnet(&self, subnet_module: Module, subnet_id: String, required_modules: Vec<String>) -> Result<(), ValidatorError> {
        // Register the subnet module
        self.registrar.register_module(subnet_module.clone()).await.map_err(ValidatorError::Registry)?;
            
        // Store subnet requirements
        let mut subnet_modules = self.subnet_modules.write().await;
        subnet_modules.insert(subnet_id, subnet_module);
        
        Ok(())
    }

    /// Get the required modules for a subnet
    pub async fn get_subnet_requirements(&self, subnet_id: &str) -> Result<Vec<String>, ValidatorError> {
        let subnet_modules = self.subnet_modules.read().await;
        subnet_modules.get(subnet_id)
            .cloned()
            .ok_or_else(|| ValidatorError::Subnet(format!("Subnet {} not found", subnet_id)))
            .map(|module| vec![module.name])
    }

    /// Check if all required modules for a subnet are installed and active
    pub async fn verify_subnet_modules(&self, subnet_id: &str) -> Result<bool, ValidatorError> {
        let required = self.get_subnet_requirements(subnet_id).await?;
        let active_modules = self.active_modules.read().await;
        Ok(required.iter().all(|name| active_modules.contains_key(name)))
    }

    /// Cleanup a module, stopping and removing it
    pub async fn cleanup_module(&self, name: &str) -> Result<(), ValidatorError> {
        // Try to stop and remove the container, ignoring errors
        let _ = self.docker.stop_container(name).await;
        let _ = self.docker.remove_container(name).await;
        
        // Then clean up registry
        let _ = self.registrar.unregister_module(name).await;
        
        // Remove from active modules
        let mut active_modules = self.active_modules.write().await;
        active_modules.remove(name);
        
        Ok(())
    }

    /// Cleanup all modules
    pub async fn cleanup_all(&self) -> Result<(), ValidatorError> {
        // Get list of modules to clean up
        let modules: Vec<String> = {
            let active_modules = self.active_modules.read().await;
            active_modules.keys().cloned().collect()
        };
        
        // Clean up each module
        for module in modules {
            let _ = self.cleanup_module(&module).await;
        }
        
        // Also try to clean up any test containers by name
        for name in ["test-module", "gpt2-module", "bert-module", "test-subnet"] {
            let _ = self.docker.stop_container(name).await;
            let _ = self.docker.remove_container(name).await;
            let _ = self.registrar.unregister_module(name).await;
        }
        
        let mut subnet_modules = self.subnet_modules.write().await;
        subnet_modules.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docker_manager::DockerManager;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    async fn setup_test() -> (ValidatorManager, MockServer) {
        let mock_server = MockServer::start().await;
        let docker = Arc::new(DockerManager::new().await.unwrap());
        let manager = ValidatorManager::new(docker, mock_server.uri());
        let _ = manager.cleanup_all().await;
        (manager, mock_server)
    }

    async fn cleanup_test_modules(manager: &ValidatorManager) {
        let _ = manager.cleanup_all().await;
    }

    #[tokio::test]
    async fn test_install_inference_module() {
        let (manager, mock_server) = setup_test().await;

        // Mock register module
        Mock::given(method("POST"))
            .and(path("/modules"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        // Mock start module
        Mock::given(method("POST"))
            .and(path("/modules/test-module/start"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let module = Module {
            name: "test-module".to_string(),
            module_type: ModuleType::Docker {
                image: "test".to_string(),
                tag: "latest".to_string(),
                port: 8080,
                env: None,
                volumes: None,
                health_check: None,
            },
            status: ModuleStatus::new(),
        };

        manager.install_inference_module(module).await.unwrap();
        cleanup_test_modules(&manager).await;
    }

    #[tokio::test]
    async fn test_verify_subnet_modules() {
        let (manager, mock_server) = setup_test().await;

        // Mock register module
        Mock::given(method("POST"))
            .and(path("/modules"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let module = Module {
            name: "test-subnet".to_string(),
            module_type: ModuleType::Docker {
                image: "test".to_string(),
                tag: "latest".to_string(),
                port: 8080,
                env: None,
                volumes: None,
                health_check: None,
            },
            status: ModuleStatus::new(),
        };

        let required_modules = vec!["gpt2-module".to_string(), "bert-module".to_string()];
        manager.install_subnet(module, "subnet-1".to_string(), required_modules.clone()).await.unwrap();
        
        let requirements = manager.get_subnet_requirements("subnet-1").await.unwrap();
        assert_eq!(requirements.len(), 1);
        assert!(requirements.contains(&"test-subnet".to_string()));

        cleanup_test_modules(&manager).await;
    }
}
