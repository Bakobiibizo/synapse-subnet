use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Invalid block time")]
    InvalidBlockTime,
    #[error("Invalid reward amount")]
    InvalidRewardAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerMetrics {
    total_blocks: u64,
    successful_attempts: u64,
    total_attempts: u64,
    total_block_time: Duration,
    rewards_earned: u64,
}

impl MinerMetrics {
    pub fn new() -> Self {
        Self {
            total_blocks: 0,
            successful_attempts: 0,
            total_attempts: 0,
            total_block_time: Duration::from_secs(0),
            rewards_earned: 0,
        }
    }

    pub fn record_block_mined(&mut self) -> Result<(), MetricsError> {
        self.total_blocks += 1;
        Ok(())
    }

    pub fn record_mining_attempt(&mut self, success: bool) -> Result<(), MetricsError> {
        self.total_attempts += 1;
        if success {
            self.successful_attempts += 1;
        }
        Ok(())
    }

    pub fn record_block_time(&mut self, time: Duration) -> Result<(), MetricsError> {
        if time.as_secs() == 0 {
            return Err(MetricsError::InvalidBlockTime);
        }
        self.total_block_time += time;
        Ok(())
    }

    pub fn record_reward(&mut self, amount: u64) -> Result<(), MetricsError> {
        if amount == 0 {
            return Err(MetricsError::InvalidRewardAmount);
        }
        self.rewards_earned += amount;
        Ok(())
    }

    pub fn total_blocks(&self) -> u64 {
        self.total_blocks
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_attempts as f64 / self.total_attempts as f64
        }
    }

    pub fn average_block_time(&self) -> Duration {
        if self.total_blocks == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(self.total_block_time.as_secs() / self.total_blocks)
        }
    }

    pub fn rewards_earned(&self) -> u64 {
        self.rewards_earned
    }

    pub fn reset(&mut self) {
        self.total_blocks = 0;
        self.successful_attempts = 0;
        self.total_attempts = 0;
        self.total_block_time = Duration::from_secs(0);
        self.rewards_earned = 0;
    }
}
