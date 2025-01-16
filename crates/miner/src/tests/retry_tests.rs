use super::*;
use crate::retry::{
    RetryConfig, RetryContext, RetryError, RetryManager, RetryableOperation,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct TestOperation {
    fail_count: Arc<AtomicU32>,
    target_success: u32,
}

#[async_trait]
impl RetryableOperation for TestOperation {
    type Output = bool;

    async fn execute(&self, context: &RetryContext) -> Result<Self::Output, RetryError> {
        let attempts = self.fail_count.fetch_add(1, Ordering::SeqCst);
        if attempts < self.target_success {
            Err(RetryError::OperationFailed("Not yet successful".to_string()))
        } else {
            Ok(true)
        }
    }
}

#[tokio::test]
async fn test_retry_config_creation() {
    let config = RetryConfig::new(PriorityLevel::High);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.base_delay, Duration::from_secs(1));

    let config = RetryConfig::new(PriorityLevel::Low);
    assert_eq!(config.max_retries, 7);
    assert_eq!(config.base_delay, Duration::from_secs(5));
}

#[tokio::test]
async fn test_successful_retry() {
    let config = RetryConfig::new(PriorityLevel::High);
    let retry_manager = RetryManager::new(config);

    let operation = TestOperation {
        fail_count: Arc::new(AtomicU32::new(0)),
        target_success: 2,
    };

    let result = retry_manager.execute(&operation).await;
    assert!(result.is_ok());
    assert_eq!(operation.fail_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_max_retries_exceeded() {
    let config = RetryConfig {
        max_retries: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        timeout: Duration::from_secs(5),
        priority: PriorityLevel::High,
    };
    let retry_manager = RetryManager::new(config);

    let operation = TestOperation {
        fail_count: Arc::new(AtomicU32::new(0)),
        target_success: 5,
    };

    let result = retry_manager.execute(&operation).await;
    assert!(matches!(result, Err(RetryError::MaxRetriesExceeded)));
    assert_eq!(operation.fail_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_timeout() {
    let config = RetryConfig {
        max_retries: 10,
        base_delay: Duration::from_secs(2),
        max_delay: Duration::from_secs(5),
        timeout: Duration::from_secs(1),
        priority: PriorityLevel::Low,
    };
    let retry_manager = RetryManager::new(config);

    let operation = TestOperation {
        fail_count: Arc::new(AtomicU32::new(0)),
        target_success: 5,
    };

    let result = retry_manager.execute(&operation).await;
    assert!(matches!(result, Err(RetryError::Timeout)));
}

#[tokio::test]
async fn test_exponential_backoff() {
    let config = RetryConfig {
        max_retries: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        timeout: Duration::from_secs(5),
        priority: PriorityLevel::Medium,
    };
    let retry_manager = RetryManager::new(config);

    let operation = TestOperation {
        fail_count: Arc::new(AtomicU32::new(0)),
        target_success: 3,
    };

    let start = std::time::Instant::now();
    let result = retry_manager.execute(&operation).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Should take at least the sum of the first three delays
    assert!(elapsed >= Duration::from_millis(100 + 150 + 225));
}

#[tokio::test]
async fn test_priority_based_delays() {
    async fn measure_retry_time(priority: PriorityLevel) -> Duration {
        let config = RetryConfig::new(priority);
        let retry_manager = RetryManager::new(config);

        let operation = TestOperation {
            fail_count: Arc::new(AtomicU32::new(0)),
            target_success: 1,
        };

        let start = std::time::Instant::now();
        let _ = retry_manager.execute(&operation).await;
        start.elapsed()
    }

    let high_priority_time = measure_retry_time(PriorityLevel::High).await;
    let low_priority_time = measure_retry_time(PriorityLevel::Low).await;

    assert!(high_priority_time < low_priority_time);
}

#[tokio::test]
async fn test_concurrent_retries() {
    let config = RetryConfig::new(PriorityLevel::High);
    let retry_manager = RetryManager::new(config);

    let fail_count = Arc::new(AtomicU32::new(0));
    let operation1 = TestOperation {
        fail_count: fail_count.clone(),
        target_success: 2,
    };
    let operation2 = TestOperation {
        fail_count: fail_count.clone(),
        target_success: 3,
    };

    let (result1, result2) = tokio::join!(
        retry_manager.execute(&operation1),
        retry_manager.execute(&operation2)
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(fail_count.load(Ordering::SeqCst) >= 5);
}
