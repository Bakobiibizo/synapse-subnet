use super::*;
use crate::status::{
    HealthStatus, NetworkStatus, StatusError, StatusManager, StatusManagerImpl, StatusUpdate,
};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[tokio::test]
async fn test_status_manager_creation() {
    let status_manager = StatusManagerImpl::new(100);
    assert!(status_manager.subscribe().await.is_ok());
}

#[tokio::test]
async fn test_status_update_broadcast() {
    let status_manager = StatusManagerImpl::new(100);
    let mut receiver1 = status_manager.subscribe().await.unwrap();
    let mut receiver2 = status_manager.subscribe().await.unwrap();

    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics: MinerMetrics::new(),
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 50.0,
            memory_usage_percent: 30.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 100,
                bandwidth_mbps: 10.0,
            },
        },
        last_error: None,
    };

    // Send update
    status_manager.send_update(update.clone()).await.unwrap();

    // Both receivers should get the update
    let received1 = receiver1.try_recv().unwrap();
    let received2 = receiver2.try_recv().unwrap();

    assert_eq!(received1.state, MinerState::Running);
    assert_eq!(received2.state, MinerState::Running);
    assert!(received1.health_status.is_healthy);
    assert!(received2.health_status.is_healthy);
}

#[tokio::test]
async fn test_connection_lifecycle() {
    let mut status_manager = StatusManagerImpl::new(100);

    // Test connection
    assert!(status_manager.connect().await.is_ok());

    // Send updates while connected
    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics: MinerMetrics::new(),
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 50.0,
            memory_usage_percent: 30.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 100,
                bandwidth_mbps: 10.0,
            },
        },
        last_error: None,
    };
    assert!(status_manager.send_update(update).await.is_ok());

    // Test disconnection
    assert!(status_manager.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_buffer_overflow() {
    let status_manager = StatusManagerImpl::new(2); // Small buffer size
    let mut receiver = status_manager.subscribe().await.unwrap();

    // Send more updates than buffer size
    for i in 0..5 {
        let update = StatusUpdate {
            timestamp: SystemTime::now(),
            state: MinerState::Running,
            metrics: MinerMetrics::new(),
            health_status: HealthStatus {
                is_healthy: true,
                cpu_usage_percent: i as f64 * 10.0,
                memory_usage_percent: 30.0,
                network_status: NetworkStatus {
                    is_connected: true,
                    latency_ms: 100,
                    bandwidth_mbps: 10.0,
                },
            },
            last_error: None,
        };
        status_manager.send_update(update).await.unwrap();
    }

    // Should still be able to receive latest updates
    let received = receiver.try_recv().unwrap();
    assert_eq!(received.health_status.cpu_usage_percent, 40.0);
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let status_manager = StatusManagerImpl::new(100);
    let mut receivers: Vec<_> = (0..10)
        .map(|_| status_manager.subscribe().await.unwrap())
        .collect();

    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics: MinerMetrics::new(),
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 50.0,
            memory_usage_percent: 30.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 100,
                bandwidth_mbps: 10.0,
            },
        },
        last_error: None,
    };

    // Send update
    status_manager.send_update(update).await.unwrap();

    // All receivers should get the update
    for receiver in &mut receivers {
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.state, MinerState::Running);
        assert!(received.health_status.is_healthy);
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mut status_manager = StatusManagerImpl::new(100);

    // Test disconnecting when not connected
    let result = status_manager.disconnect().await;
    assert!(matches!(result, Err(StatusError::ConnectionFailed(_))));

    // Connect and verify error state is cleared
    status_manager.connect().await.unwrap();
    let update = StatusUpdate {
        timestamp: SystemTime::now(),
        state: MinerState::Running,
        metrics: MinerMetrics::new(),
        health_status: HealthStatus {
            is_healthy: true,
            cpu_usage_percent: 50.0,
            memory_usage_percent: 30.0,
            network_status: NetworkStatus {
                is_connected: true,
                latency_ms: 100,
                bandwidth_mbps: 10.0,
            },
        },
        last_error: Some("Previous error".to_string()),
    };
    status_manager.send_update(update).await.unwrap();

    // Disconnect and verify error handling
    status_manager.disconnect().await.unwrap();
}
