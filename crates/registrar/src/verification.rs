use crate::module::{Module, ModuleType};
use thiserror::Error;
use url::Url;
use std::collections::HashSet;
use std::path::PathBuf;

/// Errors that can occur during module verification
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Invalid module name: {0}")]
    InvalidName(String),
    #[error("Invalid Docker image: {0}")]
    InvalidImage(String),
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid health check configuration: {0}")]
    InvalidHealthCheck(String),
    #[error("Invalid volume path: {0}")]
    InvalidVolume(String),
    #[error("Security check failed: {0}")]
    SecurityCheck(String),
}

/// Configuration for module verification
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Allowed Docker image registries
    pub allowed_registries: HashSet<String>,
    /// Allowed port ranges
    pub allowed_port_ranges: Vec<(u16, u16)>,
    /// Required environment variables
    pub required_env_vars: HashSet<String>,
    /// Allowed volume mount paths
    pub allowed_volume_paths: HashSet<String>,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            allowed_registries: HashSet::from([
                "docker.io".to_string(),
                "ghcr.io".to_string(),
            ]),
            allowed_port_ranges: vec![(1024, 65535)],
            required_env_vars: HashSet::from([
                "MODULE_NAME".to_string(),
                "MODULE_PORT".to_string(),
            ]),
            allowed_volume_paths: HashSet::from([
                "/data".to_string(),
                "/tmp".to_string(),
                "/var/log".to_string(),
            ]),
        }
    }
}

/// Module verifier that checks module configuration for compliance and security
pub struct ModuleVerifier {
    config: VerificationConfig,
}

impl ModuleVerifier {
    /// Create a new ModuleVerifier with the given configuration
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    /// Create a new ModuleVerifier with default configuration
    pub fn default() -> Self {
        Self {
            config: VerificationConfig::default(),
        }
    }

    /// Verify a module's configuration
    pub fn verify(&self, module: &Module) -> Result<(), VerificationError> {
        // Verify module name
        self.verify_name(&module.name)?;

        // Verify module type specific configuration
        match &module.module_type {
            ModuleType::Docker { 
                image, 
                tag: _, 
                port,
                env,
                volumes,
                health_check,
            } => {
                // Verify Docker image
                self.verify_docker_image(image)?;

                // Verify port
                self.verify_port(*port)?;

                // Verify environment variables
                self.verify_env_vars(env)?;

                // Verify volumes
                self.verify_volumes(volumes)?;

                // Verify health check
                self.verify_health_check(health_check)?;
            }
            ModuleType::Local { path } => {
                // Verify local path
                self.verify_local_path(path)?;
            }
        }

        Ok(())
    }

    /// Verify module name format
    fn verify_name(&self, name: &str) -> Result<(), VerificationError> {
        // Name should be lowercase alphanumeric with dashes
        if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(VerificationError::InvalidName(
                "Name must be lowercase alphanumeric with dashes".to_string(),
            ));
        }

        // Name should be between 3 and 63 characters
        if name.len() < 3 || name.len() > 63 {
            return Err(VerificationError::InvalidName(
                "Name must be between 3 and 63 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Verify Docker image format and registry
    fn verify_docker_image(&self, image: &str) -> Result<(), VerificationError> {
        // Parse image as URL to check registry
        let image_url = if image.contains("://") {
            Url::parse(image)
        } else {
            Url::parse(&format!("docker://{}", image))
        }.map_err(|_| VerificationError::InvalidImage(
            "Invalid image format".to_string(),
        ))?;

        // Check if registry is allowed
        let registry = image_url.host_str().ok_or_else(|| {
            VerificationError::InvalidImage("Missing registry".to_string())
        })?;

        if !self.config.allowed_registries.contains(registry) {
            return Err(VerificationError::InvalidImage(
                format!("Registry not allowed: {}", registry),
            ));
        }

        Ok(())
    }

    /// Verify port number is in allowed range
    fn verify_port(&self, port: u16) -> Result<(), VerificationError> {
        for (min, max) in &self.config.allowed_port_ranges {
            if port >= *min && port <= *max {
                return Ok(());
            }
        }

        Err(VerificationError::InvalidPort(port))
    }

    /// Verify required environment variables are present
    fn verify_env_vars(&self, env: &Option<std::collections::HashMap<String, String>>) 
        -> Result<(), VerificationError> 
    {
        let env = env.as_ref().ok_or_else(|| {
            VerificationError::MissingEnvVar("No environment variables defined".to_string())
        })?;

        for required_var in &self.config.required_env_vars {
            if !env.contains_key(required_var) {
                return Err(VerificationError::MissingEnvVar(
                    required_var.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Verify volume mount paths
    fn verify_volumes(&self, volumes: &Option<std::collections::HashMap<String, String>>)
        -> Result<(), VerificationError>
    {
        if let Some(volumes) = volumes {
            for (host_path, _) in volumes {
                let path = PathBuf::from(host_path);
                
                // Check if path or any parent is in allowed paths
                let mut allowed = false;
                for allowed_path in &self.config.allowed_volume_paths {
                    let allowed_path = PathBuf::from(allowed_path);
                    if path.starts_with(allowed_path) {
                        allowed = true;
                        break;
                    }
                }

                if !allowed {
                    return Err(VerificationError::InvalidVolume(
                        format!("Path not allowed: {}", host_path),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Verify health check configuration
    fn verify_health_check(&self, health_check: &Option<docker_manager::HealthCheckConfig>)
        -> Result<(), VerificationError>
    {
        if let Some(health_check) = health_check {
            // Verify test command exists
            if health_check.test.is_empty() {
                return Err(VerificationError::InvalidHealthCheck(
                    "Health check test command is empty".to_string(),
                ));
            }

            // Convert nanoseconds to seconds for validation
            let interval_secs = health_check.interval / 1_000_000_000;
            let timeout_secs = health_check.timeout / 1_000_000_000;

            // Verify reasonable intervals
            if interval_secs < 1 || interval_secs > 300 {
                return Err(VerificationError::InvalidHealthCheck(
                    "Health check interval must be between 1 and 300 seconds".to_string(),
                ));
            }

            if timeout_secs < 1 || timeout_secs > 60 {
                return Err(VerificationError::InvalidHealthCheck(
                    "Health check timeout must be between 1 and 60 seconds".to_string(),
                ));
            }

            if health_check.retries < 1 || health_check.retries > 10 {
                return Err(VerificationError::InvalidHealthCheck(
                    "Health check retries must be between 1 and 10".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Verify local module path
    fn verify_local_path(&self, path: &str) -> Result<(), VerificationError> {
        // Path should be absolute
        if !path.starts_with('/') {
            return Err(VerificationError::InvalidVolume(
                "Path must be absolute".to_string(),
            ));
        }

        // Path should exist
        if !PathBuf::from(path).exists() {
            return Err(VerificationError::InvalidVolume(
                "Path does not exist".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::ModuleStatus;

    fn create_test_module(name: &str) -> Module {
        Module {
            name: name.to_string(),
            module_type: ModuleType::Docker {
                image: "docker.io/library/nginx".to_string(),
                tag: "latest".to_string(),
                port: 8080,
                env: Some(std::collections::HashMap::from([
                    ("MODULE_NAME".to_string(), name.to_string()),
                    ("MODULE_PORT".to_string(), "8080".to_string()),
                ])),
                volumes: Some(std::collections::HashMap::from([
                    ("/data/test".to_string(), "/usr/share/nginx/html".to_string()),
                ])),
                health_check: Some(docker_manager::HealthCheckConfig {
                    test: vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), "http://localhost/health".to_string()],
                    interval: 30000000000,
                    timeout: 5000000000,
                    retries: 3,
                }),
            },
            status: ModuleStatus::new(),
        }
    }

    #[test]
    fn test_valid_module() {
        let verifier = ModuleVerifier::default();
        let module = create_test_module("test-module");
        assert!(verifier.verify(&module).is_ok());
    }

    #[test]
    fn test_invalid_name() {
        let verifier = ModuleVerifier::default();
        let module = create_test_module("TEST_MODULE");
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::InvalidName(_))
        ));
    }

    #[test]
    fn test_invalid_registry() {
        let verifier = ModuleVerifier::default();
        let mut module = create_test_module("test-module");
        if let ModuleType::Docker { ref mut image, .. } = module.module_type {
            *image = "private.registry.com/nginx".to_string();
        }
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::InvalidImage(_))
        ));
    }

    #[test]
    fn test_invalid_port() {
        let verifier = ModuleVerifier::default();
        let mut module = create_test_module("test-module");
        if let ModuleType::Docker { ref mut port, .. } = module.module_type {
            *port = 80;
        }
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::InvalidPort(_))
        ));
    }

    #[test]
    fn test_missing_env_var() {
        let verifier = ModuleVerifier::default();
        let mut module = create_test_module("test-module");
        if let ModuleType::Docker { ref mut env, .. } = module.module_type {
            env.as_mut().unwrap().remove("MODULE_NAME");
        }
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::MissingEnvVar(_))
        ));
    }

    #[test]
    fn test_invalid_volume() {
        let verifier = ModuleVerifier::default();
        let mut module = create_test_module("test-module");
        if let ModuleType::Docker { ref mut volumes, .. } = module.module_type {
            volumes.as_mut().unwrap().insert("/etc".to_string(), "/etc".to_string());
        }
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::InvalidVolume(_))
        ));
    }

    #[test]
    fn test_invalid_health_check() {
        let verifier = ModuleVerifier::default();
        let mut module = create_test_module("test-module");
        if let ModuleType::Docker { ref mut health_check, .. } = module.module_type {
            health_check.as_mut().unwrap().interval = 0;
        }
        assert!(matches!(
            verifier.verify(&module),
            Err(VerificationError::InvalidHealthCheck(_))
        ));
    }
}
