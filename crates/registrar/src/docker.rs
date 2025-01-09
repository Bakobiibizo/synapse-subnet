use crate::module::{Module, ModuleType, ModuleState, ModuleStatus, ModuleRuntime};
use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error;
use async_trait::async_trait;
use docker_manager::{ContainerManager, DockerManager, ContainerState, ContainerConfig};
use thiserror::Error;

/// Runtime implementation for Docker-based modules.
/// Handles the lifecycle of Docker containers including creation, starting, stopping,
/// and status monitoring.
pub struct DockerModuleRuntime {
    module: Module,
    manager: Arc<DockerManager>,
}

impl DockerModuleRuntime {
    pub async fn new(module: Module) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self {
            module,
            manager: Arc::new(DockerManager::new().await.map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>)?),
        })
    }

    async fn ensure_container_exists(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let ModuleType::Docker {
            image,
            tag,
            port,
            env,
            volumes,
            health_check,
        } = &self.module.module_type
        {
            let mut ports = HashMap::new();
            ports.insert("80/tcp".to_string(), port.to_string());

            let config = ContainerConfig {
                name: self.module.name.clone(),
                image: image.clone(),
                tag: tag.clone(),
                ports: Some(ports),
                env: Some(env.clone().unwrap_or_default()),
                volumes: Some(volumes.clone().unwrap_or_default()),
                health_check: health_check.clone(),
            };

            self.manager
                .create_container(config)
                .await
                .map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>)?;

            Ok(())
        } else {
            Err(Box::new(DockerRuntimeError::InvalidModuleType))
        }
    }

    async fn cleanup(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _ = self.manager
            .stop_container(&self.module.name)
            .await
            .map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>);

        let _ = self.manager
            .remove_container(&self.module.name)
            .await
            .map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>);

        Ok(())
    }
}

#[async_trait]
impl ModuleRuntime for DockerModuleRuntime {
    async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_container_exists().await?;

        self.manager
            .start_container(&self.module.name)
            .await
            .map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cleanup().await
    }

    async fn status(&self) -> Result<ModuleStatus, Box<dyn Error + Send + Sync>> {
        let container_status = self.manager.get_container_status(&self.module.name).await
            .map_err(|e| Box::new(DockerRuntimeError::Docker(e)) as Box<dyn Error + Send + Sync>)?;
        
        Ok(ModuleStatus {
            state: match container_status.state {
                ContainerState::Running => ModuleState::Running,
                ContainerState::Exited | ContainerState::Dead => ModuleState::Failed,
                _ => ModuleState::Stopped,
            },
            container_status: Some(container_status),
            error: None,
        })
    }
}

#[derive(Debug, Error)]
pub enum DockerRuntimeError {
    #[error("Docker error: {0}")]
    Docker(#[from] docker_manager::DockerError),
    #[error("Invalid module type")]
    InvalidModuleType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    use std::time::Duration;

    // Helper function to create a test module with the given name and configuration
    fn create_test_module(name: &str, port: u16) -> Module {
        Module {
            name: name.to_string(),
            module_type: ModuleType::Docker {
                image: "docker.io/library/nginx".to_string(),
                tag: "latest".to_string(),
                port,
                env: Some(HashMap::from([
                    ("NGINX_PORT".to_string(), "80".to_string()),
                    ("MODULE_NAME".to_string(), name.to_string()),
                    ("MODULE_PORT".to_string(), port.to_string()),
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
    async fn test_basic_lifecycle() {
        let module = create_test_module("test-basic-lifecycle", 8081);
        let runtime = DockerModuleRuntime::new(module.clone()).await.unwrap();

        // Start module
        runtime.start().await.unwrap();

        // Wait for container to start
        sleep(Duration::from_secs(2)).await;

        // Test module status
        let status = runtime.status().await.unwrap();
        assert_eq!(status.state, ModuleState::Running);

        // Stop module
        runtime.stop().await.unwrap();

        // Cleanup
        runtime.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_volume_management() {
        setup_test_volume().await.unwrap();
        let module = create_test_module("test-volume", 8082);
        let runtime = DockerModuleRuntime::new(module.clone()).await.unwrap();

        // Start module
        runtime.start().await.unwrap();

        // Wait for container to start
        sleep(Duration::from_secs(2)).await;

        // Test module status
        let status = runtime.status().await.unwrap();
        assert_eq!(status.state, ModuleState::Running);

        // Stop module
        runtime.stop().await.unwrap();

        // Cleanup
        runtime.cleanup().await.unwrap();
        cleanup_test_volume().await.unwrap();
    }

    #[tokio::test]
    async fn test_health_check() {
        let module = create_test_module("test-health", 8083);
        let runtime = DockerModuleRuntime::new(module.clone()).await.unwrap();

        // Start module
        runtime.start().await.unwrap();

        // Wait for container to start
        sleep(Duration::from_secs(2)).await;

        // Test module status
        let status = runtime.status().await.unwrap();
        assert_eq!(status.state, ModuleState::Running);

        // Stop module
        runtime.stop().await.unwrap();

        // Cleanup
        runtime.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Create module with invalid image
        let mut module = create_test_module("test-error", 8084);
        if let ModuleType::Docker { ref mut image, .. } = module.module_type {
            *image = "nonexistent-image:latest".to_string();
        }

        let runtime = DockerModuleRuntime::new(module).await.unwrap();

        // Starting should fail due to invalid image
        let result = runtime.start().await;
        assert!(result.is_err());
    }

    async fn setup_test_volume() -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::fs::create_dir_all("/tmp/test").await.unwrap();
        tokio::fs::write("/tmp/test/index.html", "Hello, World!").await.unwrap();
        Ok(())
    }

    async fn cleanup_test_volume() -> Result<(), Box<dyn Error + Send + Sync>> {
        tokio::fs::remove_dir_all("/tmp/test").await.unwrap();
        Ok(())
    }
}
