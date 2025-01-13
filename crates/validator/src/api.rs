use axum::{
    extract::State,
    routing::get,
    Json, Router,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;

use crate::{ValidatorError, ValidatorManager};

/// Create the API router with routes for monitoring and control
pub fn create_router(validator: Arc<ValidatorManager>) -> Router {
    router(validator)
}

/// Create the API router with routes for monitoring and control
pub fn router(validator: Arc<ValidatorManager>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .with_state(validator)
}

/// Get health status
async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok"
    }))
}

/// Get status of all modules
async fn status(
    State(validator): State<Arc<ValidatorManager>>,
) -> Response {
    match validator.get_modules().await {
        Ok(modules) => Json(json!({
            "status": "ok",
            "modules": modules
        })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "error": err.to_string()
            }))
        ).into_response()
    }
}

/// Convert validator errors to API responses
pub async fn error_response(error: ValidatorError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "status": "error",
            "error": error.to_string()
        }))
    ).into_response()
}
