//! Configuration types for module environments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a subnet module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Module information
    pub module: ModuleInfo,
    /// Environment configuration
    pub environment: EnvironmentConfig,
    /// Docker configuration
    pub docker: DockerConfig,
}

/// Basic module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Module description
    pub description: Option<String>,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Required environment variables
    pub variables: Vec<String>,
    /// Volume mappings
    pub volumes: Option<Vec<String>>,
    /// Resource limits
    pub resources: Option<ResourceLimits>,
}

/// Docker-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Docker image name
    pub image: String,
    /// Image tag
    pub tag: Option<String>,
    /// Port mappings (host:container)
    pub ports: Option<Vec<String>>,
    /// Additional Docker labels
    pub labels: Option<HashMap<String, String>>,
}

/// Resource limits for the module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (in shares)
    pub cpu: Option<f64>,
    /// Memory limit (in MB)
    pub memory: Option<u64>,
    /// Disk space limit (in MB)
    pub disk: Option<u64>,
}
