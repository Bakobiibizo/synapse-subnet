//! Miner API handlers

use super::*;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::interface::core::models::miner::*;
use crate::interface::core::services::miner::MinerService;
use std::sync::Arc;

/// Query parameters for miner operations
#[derive(Debug, Deserialize)]
pub struct MinerQuery {
    /// Optional module filter
    pub module: Option<String>,
    /// Page number for pagination
    pub page: Option<u32>,
    /// Items per page
    pub per_page: Option<u32>,
}

/// Register as a miner
///
/// POST /api/miner/register
pub async fn register_miner(
    State(state): State<Arc<dyn MinerService>>,
    Json(registration): Json<MinerRegistration>,
) -> impl IntoResponse {
    match state.register_miner(registration).await {
        Ok(status) => (StatusCode::CREATED, Json(status)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

/// Start mining for a module
///
/// POST /api/miner/modules/{name}/start
pub async fn start_mining(
    State(state): State<Arc<dyn MinerService>>,
    Path(name): Path<String>,
    Json(config): Json<MiningConfig>,
) -> impl IntoResponse {
    match state.start_mining(name, config).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

/// Stop mining for a module
///
/// POST /api/miner/modules/{name}/stop
pub async fn stop_mining(
    State(state): State<Arc<dyn MinerService>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.stop_mining(name).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

/// Get miner status
///
/// GET /api/miner/status
pub async fn get_status(
    State(state): State<Arc<dyn MinerService>>,
    Query(query): Query<MinerQuery>,
) -> impl IntoResponse {
    match state.get_status(query.module).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

/// Get mining metrics
///
/// GET /api/miner/metrics
pub async fn get_metrics(
    State(state): State<Arc<dyn MinerService>>,
    Query(query): Query<MinerQuery>,
) -> impl IntoResponse {
    match state.get_metrics(query.module).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

/// Update stake amount
///
/// PUT /api/miner/modules/{name}/stake
pub async fn update_stake(
    State(state): State<Arc<dyn MinerService>>,
    Path(name): Path<String>,
    Json(stake): Json<StakeUpdate>,
) -> impl IntoResponse {
    match state.update_stake(name, stake).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    // Mock miner service for testing
    struct MockMinerService;
    
    #[async_trait::async_trait]
    impl MinerService for MockMinerService {
        async fn register_miner(&self, _: MinerRegistration) -> Result<ModuleStatus, MinerError> {
            Ok(ModuleStatus {
                is_active: false,
                current_stake: 1000,
                uptime: 0,
                last_update: chrono::Utc::now(),
                current_metrics: None,
            })
        }

        async fn start_mining(&self, _: String, _: MiningConfig) -> Result<ModuleStatus, MinerError> {
            Ok(ModuleStatus {
                is_active: true,
                current_stake: 1000,
                uptime: 0,
                last_update: chrono::Utc::now(),
                current_metrics: None,
            })
        }

        async fn stop_mining(&self, _: String) -> Result<ModuleStatus, MinerError> {
            Ok(ModuleStatus {
                is_active: false,
                current_stake: 1000,
                uptime: 100,
                last_update: chrono::Utc::now(),
                current_metrics: None,
            })
        }

        async fn get_status(&self, _: Option<String>) -> Result<Vec<ModuleStatus>, MinerError> {
            Ok(vec![ModuleStatus {
                is_active: true,
                current_stake: 1000,
                uptime: 100,
                last_update: chrono::Utc::now(),
                current_metrics: None,
            }])
        }

        async fn get_metrics(&self, _: Option<String>) -> Result<Vec<MinerMetrics>, MinerError> {
            Ok(vec![MinerMetrics {
                total_blocks: 100,
                success_rate: 0.95,
                average_block_time: 5000,
                rewards_earned: 5000,
                last_block_timestamp: chrono::Utc::now(),
            }])
        }

        async fn update_stake(&self, _: String, update: StakeUpdate) -> Result<ModuleStatus, MinerError> {
            Ok(ModuleStatus {
                is_active: true,
                current_stake: update.new_stake,
                uptime: 100,
                last_update: chrono::Utc::now(),
                current_metrics: None,
            })
        }
    }

    fn create_test_app() -> Router {
        Router::new()
            .route("/api/miner/register", post(register_miner))
            .route("/api/miner/modules/:name/start", post(start_mining))
            .route("/api/miner/modules/:name/stop", post(stop_mining))
            .route("/api/miner/status", get(get_status))
            .route("/api/miner/metrics", get(get_metrics))
            .route("/api/miner/modules/:name/stake", put(update_stake))
            .with_state(Arc::new(MockMinerService) as Arc<dyn MinerService>)
    }

    /// Test miner registration
    #[tokio::test]
    async fn test_register_miner() {
        let app = create_test_app();
        
        let registration = MinerRegistration {
            name: "test_miner".to_string(),
            key: "test_key".to_string(),
        };

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/miner/register")
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&registration).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// Test mining start
    #[tokio::test]
    async fn test_start_mining() {
        let app = create_test_app();
        
        let config = MiningConfig {
            stake_amount: 1000,
            auto_restake: true,
        };

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/miner/modules/test_module/start")
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&config).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test metrics retrieval
    #[tokio::test]
    async fn test_get_metrics() {
        let app = create_test_app();
        
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/miner/metrics?module=test_module")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Verify response format
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response: ApiResponse<Vec<MinerMetrics>> = 
            serde_json::from_slice(&body).unwrap();
        
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }
}
