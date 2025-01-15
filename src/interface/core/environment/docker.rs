//! Docker integration for environment management

use super::{config::ModuleConfig, error::Result};
use docker_manager::ContainerConfig;
use std::collections::HashMap;

/// Create Docker container configuration from module config
pub fn create_container_config(config: &ModuleConfig) -> Result<ContainerConfig> {
    let mut env = HashMap::new();
    for var in &config.environment.variables {
        if let Ok(value) = std::env::var(var) {
            env.insert(var.clone(), value);
        }
    }

    let image = config.docker.image.clone();
    let tag = config.docker.tag.clone().unwrap_or_else(|| "latest".to_string());

    let mut ports = HashMap::new();
    if let Some(port_mappings) = &config.docker.ports {
        for mapping in port_mappings {
            let parts: Vec<&str> = mapping.split(':').collect();
            if parts.len() == 2 {
                ports.insert(
                    format!("{}/tcp", parts[1]),
                    parts[0].to_string(),
                );
            }
        }
    }

    Ok(ContainerConfig {
        name: config.module.name.clone(),
        image,
        tag,
        env: Some(env),
        ports: Some(ports),
        health_check: Some(docker_manager::HealthCheckConfig {
            test: vec![
                "CMD".to_string(),
                "curl".to_string(),
                "-f".to_string(),
                "http://localhost/health".to_string(),
            ],
            interval: 5_000_000_000, // 5s
            timeout: 3_000_000_000,  // 3s
            retries: 3,
        }),
        volumes: config.environment.volumes.clone(),
    })
}
