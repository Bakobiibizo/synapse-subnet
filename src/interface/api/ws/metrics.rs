//! WebSocket metrics handler

use super::*;
use crate::interface::core::models::{MinerMetrics, ValidatorMetrics};
use std::time::Duration;
use tokio::time;

/// Metrics update interval
const METRICS_INTERVAL: Duration = Duration::from_secs(1);

/// Start metrics broadcast loop
pub async fn start_metrics_broadcast(state: WsState, app_state: crate::interface::api::handlers::AppState) {
    let mut interval = time::interval(METRICS_INTERVAL);

    loop {
        interval.tick().await;

        // Get latest metrics
        if let Ok(metrics) = get_latest_metrics(&app_state).await {
            let _ = state.broadcast(WsMessage::Metrics(metrics)).await;
        }
    }
}

/// Get latest metrics from all sources
async fn get_latest_metrics(
    state: &crate::interface::api::handlers::AppState,
) -> Result<crate::interface::core::models::Metrics, Box<dyn std::error::Error>> {
    // Lock environment manager
    let env_manager = state.env_manager.lock().await;

    // Collect metrics from all active modules
    let mut miner_metrics = Vec::new();
    let mut validator_metrics = Vec::new();

    for module in env_manager.list_modules().await? {
        if let Ok(status) = env_manager.get_module_status(&module).await {
            if status.is_active {
                // Collect miner metrics
                if let Ok(metrics) = get_miner_metrics(&module, state).await {
                    miner_metrics.push((module.clone(), metrics));
                }

                // Collect validator metrics
                if let Ok(metrics) = get_validator_metrics(&module, state).await {
                    validator_metrics.push((module.clone(), metrics));
                }
            }
        }
    }

    Ok(crate::interface::core::models::Metrics {
        miner_metrics,
        validator_metrics,
        timestamp: chrono::Utc::now(),
    })
}

/// Get miner metrics for a module
async fn get_miner_metrics(
    module: &str,
    state: &crate::interface::api::handlers::AppState,
) -> Result<MinerMetrics, Box<dyn std::error::Error>> {
    // TODO: Implement actual metrics collection
    Ok(MinerMetrics {
        total_blocks: 100,
        success_rate: 0.95,
        average_block_time: 150,
        rewards_earned: 1000,
    })
}

/// Get validator metrics for a module
async fn get_validator_metrics(
    module: &str,
    state: &crate::interface::api::handlers::AppState,
) -> Result<ValidatorMetrics, Box<dyn std::error::Error>> {
    // TODO: Implement actual metrics collection
    Ok(ValidatorMetrics {
        total_validations: 100,
        success_rate: 0.95,
        average_response_time: 150,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    /// Test metrics collection
    #[tokio::test]
    async fn test_metrics_collection() {
        let state = crate::interface::api::handlers::AppState {
            db: crate::interface::core::db::Database::connect("sqlite::memory:").await.unwrap(),
            env_manager: std::sync::Arc::new(Mutex::new(
                crate::interface::core::environment::EnvironmentManager::new(
                    std::path::PathBuf::from("/tmp/test"),
                    std::sync::Arc::new(crate::interface::core::environment::tests::MockDockerManager::default()),
                )
                .await
                .unwrap(),
            )),
        };

        let metrics = get_latest_metrics(&state).await.unwrap();
        
        // Verify metrics format
        assert!(metrics.timestamp <= chrono::Utc::now());
        assert!(!metrics.miner_metrics.is_empty() || !metrics.validator_metrics.is_empty());
    }
}
