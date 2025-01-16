//! Environment management for subnet modules
//! 
//! Handles discovery, switching, and configuration of module environments
//! for validators and miners.

use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn, error};

mod config;
mod docker;
mod error;

pub use config::ModuleConfig;
pub use error::{Error, Result};

/// Manages module environments and configurations
pub struct EnvironmentManager {
    /// Base directory for module environments
    base_dir: PathBuf,
    /// Docker manager for container operations
    docker: Arc<dyn docker_manager::ContainerManager>,
    /// Active module environments
    active_modules: HashMap<String, ModuleEnvironment>,
}

/// Represents a module's environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEnvironment {
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Environment configuration
    pub config: ModuleConfig,
    /// Container status
    pub container_status: Option<docker_manager::ContainerStatus>,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub async fn new(
        base_dir: PathBuf,
        docker: Arc<dyn docker_manager::ContainerManager>
    ) -> Result<Self> {
        Ok(Self {
            base_dir,
            docker,
            active_modules: HashMap::new(),
        })
    }

    /// List all available module environments
    pub async fn list_environments(&self) -> Result<Vec<ModuleEnvironment>> {
        let mut environments = Vec::new();
        
        // Read module directories
        let mut entries = fs::read_dir(&self.base_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            // Load module configuration
            if let Ok(config) = self.load_module_config(&entry.path()).await {
                let name = entry.file_name().to_string_lossy().to_string();
                let container_status = self.docker
                    .get_container_status(&name)
                    .await
                    .ok();

                environments.push(ModuleEnvironment {
                    name: name.clone(),
                    version: config.version.clone(),
                    config,
                    container_status,
                });
            }
        }

        Ok(environments)
    }

    /// Switch to a module environment
    pub async fn switch_environment(&mut self, name: &str) -> Result<()> {
        // Load module configuration
        let module_dir = self.base_dir.join(name);
        let config = self.load_module_config(&module_dir).await?;

        // Load environment variables
        let env_path = module_dir.join(".env");
        if env_path.exists() {
            dotenvy::from_path(env_path)?;
        }

        // Update Docker container if needed
        let container_config = docker::create_container_config(&config)?;
        self.docker.create_container(container_config).await?;
        self.docker.start_container(name).await?;

        // Update active modules
        self.active_modules.insert(
            name.to_string(),
            ModuleEnvironment {
                name: name.to_string(),
                version: config.version.clone(),
                config,
                container_status: Some(self.docker.get_container_status(name).await?),
            },
        );

        Ok(())
    }

    /// Get current active environment
    pub fn get_active_environment(&self) -> Option<&ModuleEnvironment> {
        self.active_modules.values().next()
    }

    async fn load_module_config(&self, path: &PathBuf) -> Result<ModuleConfig> {
        let config_path = path.join("config.toml");
        let content = fs::read_to_string(config_path).await?;
        Ok(toml::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests;
