use async_trait::async_trait;
use crate::client::{Module, ModuleStatus, ClientError};

#[async_trait]
pub trait RegistrarClientTrait: Send + Sync {
    /// List all available modules
    async fn list_modules(&self) -> Result<Vec<Module>, ClientError>;
    
    /// Get a specific module by name
    async fn get_module(&self, name: &str) -> Result<Module, ClientError>;
    
    /// Create a new module
    async fn create_module(&self, module: &Module) -> Result<i64, ClientError>;
    
    /// Get the status of a module
    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError>;
    
    /// Update the status of a module
    async fn update_module_status(&self, name: &str, status: &ModuleStatus) -> Result<(), ClientError>;

    /// Start a module
    async fn start_module(&self, name: &str) -> Result<(), ClientError>;

    /// Register a new module
    async fn register_module(&self, module: &Module) -> Result<(), ClientError>;

    /// Unregister a module
    async fn unregister_module(&self, name: &str) -> Result<(), ClientError>;

    /// Register a miner
    async fn register_miner(&self, uid: &str, key: &str, name: &str) -> Result<(), ClientError>;
}
