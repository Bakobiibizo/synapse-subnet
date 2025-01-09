use async_trait::async_trait;
use std::error::Error;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use docker_manager::{ContainerStatus, HealthCheckConfig};

/// Represents a module's implementation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleType {
    /// A Docker-based module that runs in a container
    Docker {
        /// Docker image name
        image: String,
        /// Docker image tag
        tag: String,
        /// Port to expose from the container
        port: u16,
        /// Environment variables
        env: Option<HashMap<String, String>>,
        /// Volume mounts (host_path -> container_path)
        volumes: Option<HashMap<String, String>>,
        /// Health check configuration
        health_check: Option<HealthCheckConfig>,
    },
    /// A local module that runs directly on the host
    Local {
        /// Path to the module's executable or entry point
        path: String,
    },
}

/// A module represents a containerized service or local process that can be managed by the registry.
/// Each module has a unique name, type (Docker or Local), and maintains its current status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Unique name of the module
    pub name: String,
    /// Type of the module
    pub module_type: ModuleType,
    /// Current status of the module
    pub status: ModuleStatus,
}

/// Represents the current status of a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatus {
    /// Current state of the module
    pub state: ModuleState,
    /// Error message if the module is in a failed state
    pub error: Option<String>,
    /// Container status if this is a Docker module
    pub container_status: Option<ContainerStatus>,
}

impl ModuleStatus {
    /// Creates a new ModuleStatus with initial state Stopped
    pub fn new() -> Self {
        Self {
            state: ModuleState::Stopped,
            error: None,
            container_status: None,
        }
    }
}

/// Possible states a module can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// Module is stopped and not running
    Stopped,
    /// Module is currently running
    Running,
    /// Module has failed
    Failed,
}

/// Defines the runtime operations that can be performed on a module.
/// Implementations of this trait handle the actual execution of module operations
/// based on the module type (e.g., Docker containers, local processes).
#[async_trait]
pub trait ModuleRuntime: Send + Sync {
    /// Start the module
    async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Stop the module
    async fn stop(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Get the current status of the module
    async fn status(&self) -> Result<ModuleStatus, Box<dyn Error + Send + Sync>>;
}
