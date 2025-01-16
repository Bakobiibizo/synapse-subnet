use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use crate::config::PriorityLevel;

#[derive(Debug, Error)]
pub enum RetryError {
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
    #[error("Operation timeout")]
    Timeout,
    #[error("Invalid retry configuration: {0}")]
    InvalidConfig(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub timeout: Duration,
    pub priority: PriorityLevel,
}

#[derive(Debug, Clone)]
pub struct RetryContext {
    pub attempt: u32,
    pub elapsed_time: Duration,
    pub last_error: Option<String>,
}

impl RetryConfig {
    pub fn new(priority: PriorityLevel) -> Self {
        let base_config = Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            timeout: Duration::from_secs(300),
            priority,
        };

        // Adjust retry parameters based on priority
        match priority {
            PriorityLevel::High => base_config,
            PriorityLevel::Medium => Self {
                max_retries: 5,
                base_delay: Duration::from_secs(2),
                ..base_config
            },
            PriorityLevel::Low => Self {
                max_retries: 7,
                base_delay: Duration::from_secs(5),
                ..base_config
            },
            PriorityLevel::Background => Self {
                max_retries: 10,
                base_delay: Duration::from_secs(10),
                ..base_config
            },
        }
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponential_delay = self.base_delay.mul_f64(1.5f64.powi(attempt as i32));
        std::cmp::min(exponential_delay, self.max_delay)
    }
}

#[async_trait]
pub trait RetryableOperation {
    type Output;
    async fn execute(&self, context: &RetryContext) -> Result<Self::Output, RetryError>;
}

pub struct RetryManager {
    config: RetryConfig,
}

impl RetryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute<T: RetryableOperation>(
        &self,
        operation: &T,
    ) -> Result<T::Output, RetryError> {
        let start_time = std::time::Instant::now();
        let mut attempt = 0;

        loop {
            attempt += 1;
            let elapsed = start_time.elapsed();

            if elapsed > self.config.timeout {
                return Err(RetryError::Timeout);
            }

            let context = RetryContext {
                attempt,
                elapsed_time: elapsed,
                last_error: None,
            };

            match operation.execute(&context).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= self.config.max_retries {
                        return Err(RetryError::MaxRetriesExceeded);
                    }

                    let delay = self.config.calculate_delay(attempt);
                    sleep(delay).await;
                }
            }
        }
    }
}

// Example implementation of a retryable operation
pub struct MiningOperation {
    // Add fields specific to mining operation
}

#[async_trait]
impl RetryableOperation for MiningOperation {
    type Output = bool;

    async fn execute(&self, context: &RetryContext) -> Result<Self::Output, RetryError> {
        // TODO: Implement actual mining operation
        unimplemented!()
    }
}
