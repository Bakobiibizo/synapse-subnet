//! Unit tests for environment management

use super::*;
use std::sync::Arc;
use tokio::test;

/// Mock Docker manager for testing
#[derive(Default)]
struct MockDockerManager {
    containers: Arc<std::sync::Mutex<HashMap<String, docker_manager::ContainerStatus>>>,
}

#[async_trait::async_trait]
impl docker_manager::ContainerManager for MockDockerManager {
    async fn create_container(&self, config: docker_manager::ContainerConfig) -> Result<(), docker_manager::DockerError> {
        let mut containers = self.containers.lock().unwrap();
        containers.insert(config.name.clone(), docker_manager::ContainerStatus {
            state: docker_manager::ContainerState::Created,
            health: None,
            exit_code: None,
            error: None,
        });
        Ok(())
    }

    async fn start_container(&self, name: &str) -> Result<(), docker_manager::DockerError> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(status) = containers.get_mut(name) {
            status.state = docker_manager::ContainerState::Running;
            Ok(())
        } else {
            Err(docker_manager::DockerError::ContainerNotFound(name.to_string()))
        }
    }

    async fn stop_container(&self, name: &str) -> Result<(), docker_manager::DockerError> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(status) = containers.get_mut(name) {
            status.state = docker_manager::ContainerState::Exited;
            Ok(())
        } else {
            Err(docker_manager::DockerError::ContainerNotFound(name.to_string()))
        }
    }

    async fn remove_container(&self, name: &str) -> Result<(), docker_manager::DockerError> {
        let mut containers = self.containers.lock().unwrap();
        containers.remove(name)
            .ok_or_else(|| docker_manager::DockerError::ContainerNotFound(name.to_string()))?;
        Ok(())
    }

    async fn get_container_status(&self, name: &str) -> Result<docker_manager::ContainerStatus, docker_manager::DockerError> {
        let containers = self.containers.lock().unwrap();
        containers.get(name)
            .cloned()
            .ok_or_else(|| docker_manager::DockerError::ContainerNotFound(name.to_string()))
    }
}

/// Create a test environment with mock data
async fn setup_test_environment() -> (EnvironmentManager, PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // Create test module directory
    let module_dir = base_dir.join("test_module");
    tokio::fs::create_dir_all(&module_dir).await.unwrap();

    // Create test config
    let config = ModuleConfig {
        module: ModuleInfo {
            name: "test_module".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test module".to_string()),
        },
        environment: EnvironmentConfig {
            variables: vec!["TEST_VAR".to_string()],
            volumes: None,
            resources: None,
        },
        docker: DockerConfig {
            image: "test:latest".to_string(),
            tag: None,
            ports: Some(vec!["8080:80".to_string()]),
            labels: None,
        },
    };

    // Write config file
    let config_path = module_dir.join("config.toml");
    tokio::fs::write(
        &config_path,
        toml::to_string(&config).unwrap(),
    ).await.unwrap();

    // Create environment manager
    let docker = Arc::new(MockDockerManager::default());
    let manager = EnvironmentManager::new(base_dir.clone(), docker).await.unwrap();

    (manager, base_dir)
}

#[test]
async fn test_list_environments() {
    let (manager, _temp_dir) = setup_test_environment().await;
    let environments = manager.list_environments().await.unwrap();
    
    assert_eq!(environments.len(), 1);
    assert_eq!(environments[0].name, "test_module");
    assert_eq!(environments[0].version, "1.0.0");
}

#[test]
async fn test_switch_environment() {
    let (mut manager, _temp_dir) = setup_test_environment().await;
    
    // Switch to test module
    manager.switch_environment("test_module").await.unwrap();
    
    // Verify active environment
    let active = manager.get_active_environment().unwrap();
    assert_eq!(active.name, "test_module");
    
    // Verify container status
    assert!(matches!(
        active.container_status.as_ref().unwrap().state,
        docker_manager::ContainerState::Running
    ));
}

#[test]
async fn test_environment_not_found() {
    let (mut manager, _temp_dir) = setup_test_environment().await;
    
    let result = manager.switch_environment("nonexistent").await;
    assert!(matches!(result, Err(Error::IoError(_))));
}

#[test]
async fn test_invalid_config() {
    let (manager, base_dir) = setup_test_environment().await;
    
    // Create invalid config
    let module_dir = base_dir.join("invalid_module");
    tokio::fs::create_dir_all(&module_dir).await.unwrap();
    tokio::fs::write(
        module_dir.join("config.toml"),
        "invalid toml content",
    ).await.unwrap();
    
    let environments = manager.list_environments().await.unwrap();
    assert_eq!(environments.len(), 1); // Only valid module should be listed
}
