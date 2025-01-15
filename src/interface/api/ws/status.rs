//! WebSocket status handler

use super::*;
use std::time::Duration;
use tokio::time;

/// Status update interval
const STATUS_INTERVAL: Duration = Duration::from_secs(5);

/// Start status broadcast loop
pub async fn start_status_broadcast(state: WsState, app_state: crate::interface::api::handlers::AppState) {
    let mut interval = time::interval(STATUS_INTERVAL);

    loop {
        interval.tick().await;

        // Get latest status
        if let Ok(status) = get_latest_status(&app_state).await {
            let _ = state.broadcast(WsMessage::Status(status)).await;
        }
    }
}

/// Get latest status from all components
async fn get_latest_status(
    state: &crate::interface::api::handlers::AppState,
) -> Result<crate::interface::core::models::Status, Box<dyn std::error::Error>> {
    // Lock environment manager
    let env_manager = state.env_manager.lock().await;

    // Check if any modules are active
    let mut active_modules = Vec::new();
    for module in env_manager.list_modules().await? {
        if let Ok(status) = env_manager.get_module_status(&module).await {
            if status.is_active {
                active_modules.push(module);
            }
        }
    }

    Ok(crate::interface::core::models::Status {
        is_active: !active_modules.is_empty(),
        timestamp: chrono::Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    /// Test status collection
    #[tokio::test]
    async fn test_status_collection() {
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

        let status = get_latest_status(&state).await.unwrap();
        
        // Verify status format
        assert!(status.timestamp <= chrono::Utc::now());
    }
}
