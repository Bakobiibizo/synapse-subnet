use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid resource limits: {0}")]
    InvalidResourceLimits(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinerConfig {
    pub module_name: String,
    pub stake_amount: u64,
    pub auto_restake: bool,
    pub priority_level: PriorityLevel,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PriorityLevel {
    Background = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub storage_gb: u32,
}

impl ResourceLimits {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.cpu_cores <= 0.0 {
            return Err(ConfigError::InvalidResourceLimits(
                "CPU cores must be greater than 0".to_string(),
            ));
        }
        if self.memory_mb == 0 {
            return Err(ConfigError::InvalidResourceLimits(
                "Memory must be greater than 0".to_string(),
            ));
        }
        if self.storage_gb == 0 {
            return Err(ConfigError::InvalidResourceLimits(
                "Storage must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}
