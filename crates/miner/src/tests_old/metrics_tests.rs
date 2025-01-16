use super::*;
use crate::metrics::{MinerMetrics, MetricsError};
use std::time::Duration;

#[test]
fn test_miner_metrics_update() {
    let mut metrics = MinerMetrics::new();
    
    // Update block metrics
    metrics.record_block_mined().unwrap();
    assert_eq!(metrics.total_blocks(), 1);
    
    // Update success rate
    metrics.record_mining_attempt(true).unwrap();
    metrics.record_mining_attempt(false).unwrap();
    assert_eq!(metrics.success_rate(), 0.5);
    
    // Update block time
    metrics.record_block_time(Duration::from_secs(10)).unwrap();
    assert_eq!(metrics.average_block_time().as_secs(), 10);
    
    // Update rewards
    metrics.record_reward(100).unwrap();
    assert_eq!(metrics.rewards_earned(), 100);
}

#[test]
fn test_metrics_validation() {
    let mut metrics = MinerMetrics::new();
    
    // Test invalid block time
    let result = metrics.record_block_time(Duration::from_secs(0));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MetricsError::InvalidBlockTime));
    
    // Test invalid reward amount
    let result = metrics.record_reward(0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MetricsError::InvalidRewardAmount));
}

#[test]
fn test_metrics_reset() {
    let mut metrics = MinerMetrics::new();
    
    // Add some data
    metrics.record_block_mined().unwrap();
    metrics.record_mining_attempt(true).unwrap();
    metrics.record_reward(100).unwrap();
    
    // Reset metrics
    metrics.reset();
    
    // Verify reset state
    assert_eq!(metrics.total_blocks(), 0);
    assert_eq!(metrics.success_rate(), 0.0);
    assert_eq!(metrics.rewards_earned(), 0);
}
