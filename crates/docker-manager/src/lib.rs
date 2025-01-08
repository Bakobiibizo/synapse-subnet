use std::collections::HashMap;
use async_trait::async_trait;
use bollard::{
    container::{
        Config, CreateContainerOptions, ListContainersOptions,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
        InspectContainerOptions,
    },
    Docker,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("Container not found: {0}")]
    ContainerNotFound(String),
    #[error("Container already exists: {0}")]
    ContainerExists(String),
    #[error("Invalid container state: {0}")]
    InvalidState(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    pub tag: String,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<HashMap<String, String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub health_check: Option<HealthCheckConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub test: Vec<String>,
    pub interval: u64,
    pub timeout: u64,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    pub state: ContainerState,
    pub health: Option<String>,
    pub exit_code: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

#[async_trait]
pub trait ContainerManager: Send + Sync {
    /// Create a new container
    async fn create_container(&self, config: ContainerConfig) -> Result<(), DockerError>;

    /// Start a container
    async fn start_container(&self, name: &str) -> Result<(), DockerError>;

    /// Stop a container
    async fn stop_container(&self, name: &str) -> Result<(), DockerError>;

    /// Remove a container
    async fn remove_container(&self, name: &str) -> Result<(), DockerError>;

    /// Get container status
    async fn get_container_status(&self, name: &str) -> Result<ContainerStatus, DockerError>;

    /// List all containers
    async fn list_containers(&self) -> Result<Vec<ContainerStatus>, DockerError>;
}

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub async fn new() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    async fn get_container_id(&self, name: &str) -> Result<String, DockerError> {
        let mut filters = HashMap::new();
        filters.insert(String::from("name"), vec![name.to_string()]);

        let options = Some(ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        });

        let containers = self.docker.list_containers(options).await?;
        containers
            .first()
            .map(|c| c.id.clone().unwrap_or_default())
            .ok_or_else(|| DockerError::ContainerNotFound(name.to_string()))
    }
}

#[async_trait]
impl ContainerManager for DockerManager {
    async fn create_container(&self, config: ContainerConfig) -> Result<(), DockerError> {
        // Check if container already exists
        let mut filters = HashMap::new();
        filters.insert(String::from("name"), vec![config.name.to_string()]);

        let options = Some(ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        });
        let containers = self.docker.list_containers(options).await?;
        if !containers.is_empty() {
            return Err(DockerError::ContainerExists(config.name));
        }

        // Prepare container configuration
        let mut container_config = Config {
            image: Some(format!("{}:{}", config.image, config.tag)),
            ..Default::default()
        };

        // Add environment variables
        if let Some(env) = config.env {
            container_config.env = Some(
                env.into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect(),
            );
        }

        // Add port bindings
        if let Some(ports) = config.ports {
            let mut exposed_ports = HashMap::new();
            let mut port_bindings = HashMap::new();
            for (container_port, host_port) in ports {
                exposed_ports.insert(container_port.clone(), HashMap::new());
                port_bindings.insert(
                    container_port,
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(host_port),
                    }]),
                );
            }
            container_config.exposed_ports = Some(exposed_ports);
            container_config.host_config = Some(bollard::models::HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            });
        }

        // Add health check
        if let Some(health) = config.health_check {
            container_config.healthcheck = Some(bollard::models::HealthConfig {
                test: Some(health.test),
                interval: Some(health.interval as i64),
                timeout: Some(health.timeout as i64),
                retries: Some(health.retries as i64),
                ..Default::default()
            });
        }

        // Create container
        let options = CreateContainerOptions {
            name: &config.name,
            platform: None,
        };
        self.docker.create_container(Some(options), container_config).await?;
        Ok(())
    }

    async fn start_container(&self, name: &str) -> Result<(), DockerError> {
        let id = self.get_container_id(name).await?;
        self.docker
            .start_container(&id, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    async fn stop_container(&self, name: &str) -> Result<(), DockerError> {
        let id = self.get_container_id(name).await?;
        self.docker
            .stop_container(&id, None::<StopContainerOptions>)
            .await?;
        Ok(())
    }

    async fn remove_container(&self, name: &str) -> Result<(), DockerError> {
        let id = self.get_container_id(name).await?;
        let options = Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        });
        self.docker.remove_container(&id, options).await?;
        Ok(())
    }

    async fn get_container_status(&self, name: &str) -> Result<ContainerStatus, DockerError> {
        let id = self.get_container_id(name).await?;
        let details = self.docker
            .inspect_container(&id, None::<InspectContainerOptions>)
            .await?;

        let state = details.state.unwrap_or_default();
        let status = ContainerStatus {
            state: match state.status.map(|s| s.to_string()).unwrap_or_default().as_str() {
                "created" => ContainerState::Created,
                "running" => ContainerState::Running,
                "paused" => ContainerState::Paused,
                "restarting" => ContainerState::Restarting,
                "removing" => ContainerState::Removing,
                "exited" => ContainerState::Exited,
                "dead" => ContainerState::Dead,
                _ => ContainerState::Dead,
            },
            health: state.health.and_then(|h| h.status).map(|s| s.to_string()),
            exit_code: state.exit_code,
            error: state.error,
        };

        Ok(status)
    }

    async fn list_containers(&self) -> Result<Vec<ContainerStatus>, DockerError> {
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });
        let containers = self.docker.list_containers(options).await?;

        let mut statuses = Vec::new();
        for container in containers {
            if let Some(id) = container.id {
                let details = self.docker
                    .inspect_container(&id, None::<InspectContainerOptions>)
                    .await?;

                let state = details.state.unwrap_or_default();
                statuses.push(ContainerStatus {
                    state: match state.status.map(|s| s.to_string()).unwrap_or_default().as_str() {
                        "created" => ContainerState::Created,
                        "running" => ContainerState::Running,
                        "paused" => ContainerState::Paused,
                        "restarting" => ContainerState::Restarting,
                        "removing" => ContainerState::Removing,
                        "exited" => ContainerState::Exited,
                        "dead" => ContainerState::Dead,
                        _ => ContainerState::Dead,
                    },
                    health: state.health.and_then(|h| h.status).map(|s| s.to_string()),
                    exit_code: state.exit_code,
                    error: state.error,
                });
            }
        }

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_container_lifecycle() {
        let manager = DockerManager::new().await.unwrap();

        // Create container config
        let config = ContainerConfig {
            name: "test-nginx".to_string(),
            image: "nginx".to_string(),
            tag: "latest".to_string(),
            env: None,
            ports: Some(HashMap::from([
                ("80/tcp".to_string(), "8080".to_string()),
            ])),
            health_check: Some(HealthCheckConfig {
                test: vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), "http://localhost/".to_string()],
                interval: 5000000000, // 5s
                timeout: 3000000000,  // 3s
                retries: 3,
            }),
            volumes: None,
        };

        // Test container creation
        manager.create_container(config.clone()).await.unwrap();

        // Test container start
        manager.start_container(&config.name).await.unwrap();

        // Wait for container to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Test container status
        let status = manager.get_container_status(&config.name).await.unwrap();
        assert_eq!(status.state, ContainerState::Running);

        // Test container stop
        manager.stop_container(&config.name).await.unwrap();

        // Test container removal
        manager.remove_container(&config.name).await.unwrap();

        // Verify container is gone
        assert!(manager.get_container_status(&config.name).await.is_err());
    }
}
