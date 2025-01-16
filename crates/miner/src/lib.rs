//! Miner implementation for the Synapse Subnet project.
//! 
//! This crate provides the miner functionality for executing inference
//! requests using Ollama models.
//!
//! Required Features and Components:
//! 1. Ollama Integration
//!    - Model management and versioning
//!    - Model loading and unloading
//!    - Model configuration handling
//!
//! 2. Inference Request Handling
//!    - Request queue management
//!    - Priority handling
//!    - Request validation
//!    - Result formatting
//!
//! 3. Resource Management
//!    - GPU/CPU utilization monitoring
//!    - Memory usage tracking
//!    - Network bandwidth monitoring
//!    - Resource allocation
//!
//! 4. Performance Metrics
//!    - Latency tracking
//!    - Throughput monitoring
//!    - Error rate tracking
//!    - Resource utilization metrics
//!
//! 5. Fault Tolerance
//!    - Error handling and recovery
//!    - Graceful degradation
//!    - Automatic restart mechanisms
//!    - State recovery
//!
//! Key Interfaces Required:
//! - OllamaInterface: For model management
//! - ValidatorInterface: For result submission
//! - MetricsInterface: For performance monitoring
//! - StorageInterface: For state persistence

pub mod config;
pub mod metrics;
pub mod miner;

pub use config::{MinerConfig, ResourceLimits, PriorityLevel, ConfigError};
pub use metrics::{MinerMetrics, MetricsError};
pub use miner::{Miner, MinerInterface, MinerState, MinerError};

#[cfg(test)]
mod tests {
    use super::*;
    
    mod config_tests;
    mod metrics_tests;
    mod miner_tests;
}
