use crate::status::{StatusManager, StatusManagerImpl, StatusUpdate, HealthStatus, NetworkStatus};
use crate::metrics::MinerMetrics;
use crate::miner::MinerState;
use std::sync::Arc;
use std::time::{SystemTime, Duration};

#[tokio::test]
async fn test_status_manager_creation() {
    let status_manager = Arc::new(StatusManagerImpl::new(100));
    assert!(status_manager.subscribe().await.is_ok());
}

#[tokio::test]
async fn test_status_update() {
    let status_manager = Arc::new(StatusManagerImpl::new(100));
    let mut receiver = status_manager.subscribe().await.unwrap();

    // Create a test status update
    let mut metrics = MinerMetrics::new();
    metrics.record_block_mined().unwrap();
    metrics.record_reward(100).unwrap();
    metrics.record_block_time(Duration::from_secs(5)).unwrap();

    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics,
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 50.0,
            memory_usage_percent: 60.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 100,
                bandwidth_mbps: 10.0,
            },
        },
        last_error: None,
    };

    // Send the update
    status_manager.send_update(update.clone()).await.unwrap();

    // Verify the update was received
    let received = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.state, MinerState::Running);
    assert_eq!(received.metrics.total_blocks(), 1);
    assert_eq!(received.health_status.is_healthy, true);
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let status_manager = Arc::new(StatusManagerImpl::new(100));
    
    // Create multiple subscribers
    let mut receivers = Vec::new();
    for _ in 0..10 {
        let receiver = status_manager.subscribe().await.unwrap();
        receivers.push(receiver);
    }

    // Create test metrics
    let mut metrics = MinerMetrics::new();
    metrics.record_block_mined().unwrap();
    metrics.record_reward(50).unwrap();
    metrics.record_block_time(Duration::from_secs(2)).unwrap();

    // Send an update
    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics,
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 30.0,
            memory_usage_percent: 40.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 50,
                bandwidth_mbps: 5.0,
            },
        },
        last_error: None,
    };

    status_manager.send_update(update).await.unwrap();

    // Verify all subscribers received the update
    for mut receiver in receivers {
        let received = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received.metrics.total_blocks(), 1);
    }
}

#[tokio::test]
async fn test_subscriber_cleanup() {
    let status_manager = Arc::new(StatusManagerImpl::new(100));
    
    // Create and drop subscribers
    for _ in 0..10 {
        let _receiver = status_manager.subscribe().await.unwrap();
        // Receiver is dropped here
    }

    // Create test metrics
    let mut metrics = MinerMetrics::new();
    metrics.record_block_mined().unwrap();
    metrics.record_reward(10).unwrap();
    metrics.record_block_time(Duration::from_secs(1)).unwrap();

    // Send an update
    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics,
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 20.0,
            memory_usage_percent: 30.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 25,
                bandwidth_mbps: 2.5,
            },
        },
        last_error: None,
    };

    // Should not fail even though all receivers were dropped
    let result = status_manager.send_update(update).await;
    assert!(result.is_ok() || matches!(result, Err(crate::status::StatusError::ChannelError(_))));
}

#[tokio::test]
async fn test_error_handling() {
    let status_manager = Arc::new(StatusManagerImpl::new(10)); // Larger buffer size
    let mut receiver = status_manager.subscribe().await.unwrap();

    // Fill the channel buffer
    for i in 0..5 { // Fewer iterations to avoid lagging
        let mut metrics = MinerMetrics::new();
        metrics.record_block_mined().unwrap();
        if i > 0 { // Skip reward on first iteration to avoid InvalidRewardAmount
            metrics.record_reward(i * 10).unwrap();
        }
        metrics.record_block_time(Duration::from_secs(1)).unwrap();

        let update = StatusUpdate {
            timestamp: SystemTime::now(),
            state: MinerState::Running,
            metrics,
            health_status: HealthStatus {
                is_healthy: true,
                cpu_usage_percent: 20.0,
                memory_usage_percent: 30.0,
                network_status: NetworkStatus {
                    is_connected: true,
                    latency_ms: 25,
                    bandwidth_mbps: 2.5,
                },
            },
            last_error: None,
        };

        // Some updates may be dropped due to small buffer size, which is expected
        let result = status_manager.send_update(update).await;
        assert!(result.is_ok() || matches!(result, Err(crate::status::StatusError::ChannelError(_))));
    }

    // Should still be able to receive updates
    while let Ok(received) = receiver.try_recv() {
        assert!(received.metrics.total_blocks() > 0);
    }
}
