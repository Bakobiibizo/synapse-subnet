//! Core error types for the interface

use thiserror::Error;

/// Core error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Chain error: {0}")]
    Chain(String),

    #[error("Rate limit error: {0}")]
    RateLimit(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry mechanism for fallible operations
pub async fn retry<T, F, Fut>(
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay_ms;
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempts += 1;

                if attempts >= config.max_retries {
                    return Err(err);
                }

                match &err {
                    // Retry on specific errors
                    Error::Network(_) | Error::Chain(_) | Error::RateLimit(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        delay = std::cmp::min(
                            (delay as f64 * config.backoff_multiplier) as u64,
                            config.max_delay_ms,
                        );
                    }
                    // Don't retry on other errors
                    _ => return Err(err),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let operation = || async move {
            let current = attempts_clone.fetch_add(1, Ordering::SeqCst);
            if current < 2 {
                Err(Error::Network("Test error".into()))
            } else {
                Ok(42)
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry(operation, config).await;
        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_failure() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let operation = || async move {
            attempts_clone.fetch_add(1, Ordering::SeqCst);
            Err(Error::Network("Test error".into()))
        };

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry(operation, config).await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retryable() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let operation = || async move {
            attempts_clone.fetch_add(1, Ordering::SeqCst);
            Err(Error::Auth("Test error".into()))
        };

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry(operation, config).await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
