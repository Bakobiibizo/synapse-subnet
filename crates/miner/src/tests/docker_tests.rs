use crate::docker::{ContainerConfig, ContainerStats, DockerError, DockerManagerImpl, DockerManager};
use crate::config::ResourceLimits;
use std::time::Duration;

#[tokio::test]
async fn test_docker_manager_creation() {
    let docker_manager = DockerManagerImpl::new().expect("Failed to create Docker manager");
    // No is_connected method, just verify we can create the manager
}

#[tokio::test]
async fn test_container_lifecycle() {
    let docker_manager = DockerManagerImpl::new().expect("Failed to create Docker manager");
    
    // Create container
    let config = ContainerConfig {
        image: "ubuntu:latest".to_string(),
        command: vec!["sleep".to_string(), "1".to_string()],
        environment: vec!["TEST=true".to_string()],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 512,
            storage_gb: 1,
        },
    };

    let container_id = docker_manager.create_container(&config).await.unwrap();
    assert!(!container_id.is_empty());

    // Start container
    docker_manager.start_container(&container_id).await.unwrap();

    // Get container logs
    let logs = docker_manager.get_container_logs(&container_id).await.unwrap();
    assert!(!logs.is_empty());

    // Get container stats
    let stats = docker_manager.get_container_stats(&container_id).await.unwrap();
    assert!(stats.cpu_usage > 0.0);
    assert!(stats.memory_usage > 0);

    // Stop container
    docker_manager.stop_container(&container_id).await.unwrap();

    // Remove container
    docker_manager.remove_container(&container_id).await.unwrap();
}

#[tokio::test]
async fn test_invalid_container_operations() {
    let docker_manager = DockerManagerImpl::new().expect("Failed to create Docker manager");

    // Try to start non-existent container
    let result = docker_manager.start_container("invalid_id").await;
    assert!(matches!(result, Err(DockerError::ContainerStartFailed(_))));

    // Try to create container with invalid image
    let config = ContainerConfig {
        image: "invalid:latest".to_string(),
        command: vec![],
        environment: vec![],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 512,
            storage_gb: 1,
        },
    };

    let result = docker_manager.create_container(&config).await;
    assert!(matches!(result, Err(DockerError::ContainerCreationFailed(_))));
}

#[tokio::test]
async fn test_resource_limits() {
    let docker_manager = DockerManagerImpl::new().expect("Failed to create Docker manager");

    // Create container with resource limits
    let config = ContainerConfig {
        image: "ubuntu:latest".to_string(),
        command: vec!["stress".to_string(), "--cpu".to_string(), "2".to_string(), "--timeout".to_string(), "5".to_string()],
        environment: vec![],
        resource_limits: ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 256,
            storage_gb: 1,
        },
    };

    let container_id = docker_manager.create_container(&config).await.unwrap();
    docker_manager.start_container(&container_id).await.unwrap();

    // Wait for container to run
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check stats
    let stats = docker_manager.get_container_stats(&container_id).await.unwrap();
    assert!(stats.cpu_usage <= 100.0); // Should be limited to 1 core
    assert!(stats.memory_usage <= 256 * 1024 * 1024); // Should be limited to 256MB

    // Cleanup
    docker_manager.stop_container(&container_id).await.unwrap();
    docker_manager.remove_container(&container_id).await.unwrap();
}

#[tokio::test]
async fn test_container_cleanup() {
    let docker_manager = DockerManagerImpl::new().expect("Failed to create Docker manager");

    // Create multiple containers
    let mut container_ids = Vec::new();
    for _ in 0..3 {
        let config = ContainerConfig {
            image: "ubuntu:latest".to_string(),
            command: vec!["sleep".to_string(), "1".to_string()],
            environment: vec![],
            resource_limits: ResourceLimits {
                cpu_cores: 1.0,
                memory_mb: 256,
                storage_gb: 1,
            },
        };

        let container_id = docker_manager.create_container(&config).await.unwrap();
        container_ids.push(container_id);
    }

    // Start all containers
    for container_id in &container_ids {
        docker_manager.start_container(container_id).await.unwrap();
    }

    // Clean up all containers
    for container_id in &container_ids {
        docker_manager.stop_container(container_id).await.unwrap();
        docker_manager.remove_container(container_id).await.unwrap();
    }
}
