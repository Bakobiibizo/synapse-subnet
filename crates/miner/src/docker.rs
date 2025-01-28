use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions, LogsOptions, StatsOptions};
use bollard::models::HostConfig;
use bollard::errors::Error as BollardError;
use thiserror::Error;
use crate::config::ResourceLimits;
use futures_util::StreamExt;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Container creation failed: {0}")]
    ContainerCreationFailed(String),
    #[error("Container start failed: {0}")]
    ContainerStartFailed(String),
    #[error("Container stop failed: {0}")]
    ContainerStopFailed(String),
    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),
    #[error("Docker API error: {0}")]
    DockerApiError(#[from] bollard::errors::Error),
}

pub struct ContainerConfig {
    pub image: String,
    pub command: Vec<String>,
    pub environment: Vec<String>,
    pub resource_limits: ResourceLimits,
}

#[async_trait]
pub trait DockerManager {
    async fn create_container(&self, config: &ContainerConfig) -> Result<String, DockerError>;
    async fn start_container(&self, container_id: &str) -> Result<(), DockerError>;
    async fn stop_container(&self, container_id: &str) -> Result<(), DockerError>;
    async fn remove_container(&self, container_id: &str) -> Result<(), DockerError>;
    async fn get_container_logs(&self, container_id: &str) -> Result<Vec<String>, DockerError>;
    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats, DockerError>;
}

#[derive(Debug, Clone)]
pub struct ContainerStats {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_rx: u64,
    pub network_tx: u64,
}

pub struct DockerManagerImpl {
    docker: Docker,
}

impl DockerManagerImpl {
    pub fn new() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| DockerError::DockerApiError(e))?;
        Ok(Self { docker })
    }
}

#[async_trait]
impl DockerManager for DockerManagerImpl {
    async fn create_container(&self, config: &ContainerConfig) -> Result<String, DockerError> {
        // Convert resource limits to host config
        let host_config = HostConfig {
            cpu_shares: Some((config.resource_limits.cpu_cores * 1024.0) as i64),
            memory: Some((config.resource_limits.memory_mb as i64) * 1024 * 1024),
            memory_swap: Some((config.resource_limits.memory_mb as i64) * 1024 * 1024), // Same as memory limit (no swap)
            ..Default::default()
        };

        // Create container config
        let container_config = Config {
            image: Some(config.image.clone()),
            cmd: Some(config.command.clone()),
            env: Some(config.environment.clone()),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Create container
        let options = CreateContainerOptions {
            name: "",
            platform: None,
        };

        let response = self.docker.create_container(Some(options), container_config)
            .await
            .map_err(|e| DockerError::ContainerCreationFailed(e.to_string()))?;

        Ok(response.id)
    }

    async fn start_container(&self, container_id: &str) -> Result<(), DockerError> {
        self.docker.start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| DockerError::ContainerStartFailed(e.to_string()))
    }

    async fn stop_container(&self, container_id: &str) -> Result<(), DockerError> {
        self.docker.stop_container(container_id, None::<StopContainerOptions>)
            .await
            .map_err(|e| DockerError::ContainerStopFailed(e.to_string()))
    }

    async fn remove_container(&self, container_id: &str) -> Result<(), DockerError> {
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        self.docker.remove_container(container_id, Some(options))
            .await
            .map_err(|e| DockerError::DockerApiError(e))
    }

    async fn get_container_logs(&self, container_id: &str) -> Result<Vec<String>, DockerError> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        };

        let mut logs = Vec::new();
        let mut stream = self.docker.logs(container_id, Some(options));

        while let Some(log) = stream.next().await {
            match log {
                Ok(log) => logs.push(log.to_string()),
                Err(e) => return Err(DockerError::DockerApiError(e)),
            }
        }

        Ok(logs)
    }

    async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats, DockerError> {
        let options = StatsOptions {
            stream: false,
            ..Default::default()
        };

        let mut stream = self.docker.stats(container_id, Some(options));

        if let Some(stats_result) = stream.next().await {
            match stats_result {
                Ok(stats) => {
                    let cpu_usage = match (&stats.cpu_stats, &stats.precpu_stats) {
                        (Some(cpu_stats), Some(precpu_stats)) => {
                            let cpu_delta = cpu_stats.cpu_usage.total_usage as f64 - precpu_stats.cpu_usage.total_usage as f64;
                            let system_delta = cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
                            if system_delta > 0.0 && cpu_delta > 0.0 {
                                (cpu_delta / system_delta) * 100.0 * (cpu_stats.online_cpus.unwrap_or(1) as f64)
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };

                    let memory_usage = stats.memory_stats.usage.unwrap_or(0);
                    let network_stats = stats.networks.unwrap_or_default();
                    let (mut rx_bytes, mut tx_bytes) = (0, 0);
                    for (_, net_stats) in network_stats {
                        rx_bytes += net_stats.rx_bytes;
                        tx_bytes += net_stats.tx_bytes;
                    }

                    Ok(ContainerStats {
                        cpu_usage,
                        memory_usage,
                        network_rx: rx_bytes,
                        network_tx: tx_bytes,
                    })
                }
                Err(e) => Err(DockerError::DockerApiError(e)),
            }
        } else {
            Err(DockerError::DockerApiError(BollardError::IOError {
                err: std::io::Error::new(std::io::ErrorKind::Other, "No stats received"),
            }))
        }
    }
}
