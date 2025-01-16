use super::*;
use crate::docker::{ContainerConfig, ContainerStats, DockerError, DockerManager, DockerManagerImpl};
use std::time::Duration;

#[tokio::test]
async fn test_docker_manager_creation() {
    let docker_manager = DockerManagerImpl::new();
    assert!(docker_manager.is_ok());
}

#[tokio::test]
async fn test_container_lifecycle() {
    let docker_manager = DockerManagerImpl::new().unwrap();
    let config = ContainerConfig {
        image: "ubuntu:latest".to_string(),
        command: vec!["echo".to_string(), "hello".to_string()],
        environment: vec!["TEST=true".to_string()],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 512,
            storage_gb: 1,
        },
    };

    // Create container
    let container_id = docker_manager.create_container(&config).await.unwrap();
    assert!(!container_id.is_empty());

    // Start container
    docker_manager.start_container(&container_id).await.unwrap();

    // Get container logs
    let logs = docker_manager.get_container_logs(&container_id).await.unwrap();
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|log| log.contains("hello")));

    // Get container stats
    let stats = docker_manager.get_container_stats(&container_id).await.unwrap();
    assert!(stats.cpu_usage >= 0.0);
    assert!(stats.memory_usage > 0);

    // Stop container
    docker_manager.stop_container(&container_id).await.unwrap();

    // Remove container
    docker_manager.remove_container(&container_id).await.unwrap();
}

#[tokio::test]
async fn test_resource_limits() {
    let docker_manager = DockerManagerImpl::new().unwrap();
    let config = ContainerConfig {
        image: "ubuntu:latest".to_string(),
        command: vec!["stress".to_string(), "--cpu", "2"],
        environment: vec![],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 256,
            storage_gb: 1,
        },
    };

    // Create and start container
    let container_id = docker_manager.create_container(&config).await.unwrap();
    docker_manager.start_container(&container_id).await.unwrap();

    // Wait for stats to stabilize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check resource usage
    let stats = docker_manager.get_container_stats(&container_id).await.unwrap();
    assert!(stats.cpu_usage <= 100.0); // Should be limited to 1 core
    assert!(stats.memory_usage <= 256 * 1024 * 1024); // Should be limited to 256MB

    // Cleanup
    docker_manager.stop_container(&container_id).await.unwrap();
    docker_manager.remove_container(&container_id).await.unwrap();
}

#[tokio::test]
async fn test_invalid_image() {
    let docker_manager = DockerManagerImpl::new().unwrap();
    let config = ContainerConfig {
        image: "nonexistent:latest".to_string(),
        command: vec![],
        environment: vec![],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 256,
            storage_gb: 1,
        },
    };

    let result = docker_manager.create_container(&config).await;
    assert!(matches!(result, Err(DockerError::ContainerCreationFailed(_))));
}

#[tokio::test]
async fn test_invalid_container_operations() {
    let docker_manager = DockerManagerImpl::new().unwrap();
    let invalid_id = "nonexistent_container";

    // Try to start non-existent container
    let result = docker_manager.start_container(invalid_id).await;
    assert!(matches!(result, Err(DockerError::ContainerStartFailed(_))));

    // Try to stop non-existent container
    let result = docker_manager.stop_container(invalid_id).await;
    assert!(matches!(result, Err(DockerError::ContainerStopFailed(_))));

    // Try to get logs from non-existent container
    let result = docker_manager.get_container_logs(invalid_id).await;
    assert!(result.is_err());

    // Try to get stats from non-existent container
    let result = docker_manager.get_container_stats(invalid_id).await;
    assert!(result.is_err());
}
