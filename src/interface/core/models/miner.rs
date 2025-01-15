use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Priority level for mining operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource limits for miner containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub storage_gb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        }
    }
}

/// Miner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
    pub module_name: String,
    pub stake_amount: u64,
    pub auto_restake: bool,
    pub priority_level: PriorityLevel,
    pub resource_limits: ResourceLimits,
}

/// Mining metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerMetrics {
    pub total_blocks: u64,
    pub success_rate: f64,
    pub average_block_time: u64,
    pub rewards_earned: u64,
    pub last_block_timestamp: DateTime<Utc>,
}

/// Module status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatus {
    pub is_active: bool,
    pub current_stake: u64,
    pub uptime: u64,
    pub last_update: DateTime<Utc>,
    pub current_metrics: Option<MinerMetrics>,
}

/// Miner registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerRegistration {
    pub module_name: String,
    pub initial_stake: u64,
    pub config: MinerConfig,
}

/// Mining configuration update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub auto_restake: Option<bool>,
    pub priority_level: Option<PriorityLevel>,
    pub resource_limits: Option<ResourceLimits>,
}

/// Stake update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeUpdate {
    pub new_stake: u64,
    pub auto_restake: Option<bool>,
}

/// Error types specific to mining operations
#[derive(Debug, thiserror::Error)]
pub enum MinerError {
    #[error("Invalid stake amount: {0}")]
    InvalidStake(String),
    
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    
    #[error("Container error: {0}")]
    ContainerError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceExceeded(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}
