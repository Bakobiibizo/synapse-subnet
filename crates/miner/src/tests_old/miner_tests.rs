use super::*;
use crate::miner::{Miner, MinerError, MinerState};
use tokio_test::block_on;

#[test]
fn test_miner_creation() {
    let config = MinerConfig {
        module_name: "test_module".to_string(),
        stake_amount: 1000,
        auto_restake: true,
        priority_level: PriorityLevel::High,
        resource_limits: ResourceLimits {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        },
    };

    let miner = Miner::new(config.clone());
    assert_eq!(miner.config(), &config);
    assert_eq!(miner.state(), MinerState::Initialized);
}

#[tokio::test]
async fn test_miner_lifecycle() {
    let config = MinerConfig {
        module_name: "test_module".to_string(),
        stake_amount: 1000,
        auto_restake: true,
        priority_level: PriorityLevel::High,
        resource_limits: ResourceLimits {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        },
    };

    let mut miner = Miner::new(config);
    
    // Test start
    miner.start().await.unwrap();
    assert_eq!(miner.state(), MinerState::Running);
    
    // Test pause
    miner.pause().await.unwrap();
    assert_eq!(miner.state(), MinerState::Paused);
    
    // Test resume
    miner.resume().await.unwrap();
    assert_eq!(miner.state(), MinerState::Running);
    
    // Test stop
    miner.stop().await.unwrap();
    assert_eq!(miner.state(), MinerState::Stopped);
}

#[test]
fn test_miner_metrics() {
    let config = MinerConfig {
        module_name: "test_module".to_string(),
        stake_amount: 1000,
        auto_restake: true,
        priority_level: PriorityLevel::High,
        resource_limits: ResourceLimits {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        },
    };

    let miner = Miner::new(config);
    let metrics = miner.metrics();
    
    assert_eq!(metrics.total_blocks(), 0);
    assert_eq!(metrics.success_rate(), 0.0);
    assert_eq!(metrics.rewards_earned(), 0);
}

#[tokio::test]
async fn test_invalid_state_transitions() {
    let config = MinerConfig {
        module_name: "test_module".to_string(),
        stake_amount: 1000,
        auto_restake: true,
        priority_level: PriorityLevel::High,
        resource_limits: ResourceLimits {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        },
    };

    let mut miner = Miner::new(config);
    
    // Cannot pause when not running
    let result = miner.pause().await;
    assert!(matches!(result.unwrap_err(), MinerError::InvalidStateTransition));
    
    // Cannot resume when not paused
    let result = miner.resume().await;
    assert!(matches!(result.unwrap_err(), MinerError::InvalidStateTransition));
    
    // Start the miner
    miner.start().await.unwrap();
    
    // Cannot start when already running
    let result = miner.start().await;
    assert!(matches!(result.unwrap_err(), MinerError::InvalidStateTransition));
}
