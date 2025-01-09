//! Registrar API implementation for the Synapse Subnet project.
//!
//! This crate provides the REST API interface for the module registry system.
//!
//! Required Features and Components:
//! 1. REST API Endpoints
//!    - GET /modules - List all modules
//!    - POST /modules - Create new module
//!    - GET /modules/:name - Get module details
//!    - PUT /modules/:name - Update module
//!    - DELETE /modules/:name - Remove module
//!    - GET /modules/:name/status - Get status
//!    - PUT /modules/:name/status - Update status
//!    - POST /modules/:name/start - Start module
//!
//! 2. Request/Response Handling
//!    - Input validation
//!    - Error handling
//!    - Response formatting
//!    - Status codes
//!
//! 3. Authentication & Authorization
//!    - API key validation
//!    - Role-based access
//!    - Rate limiting
//!
//! 4. Monitoring & Logging
//!    - Request logging
//!    - Performance metrics
//!    - Error tracking
//!
//! Key Interfaces Required:
//! - RegistryInterface: For module management
//! - AuthInterface: For authentication
//! - MetricsInterface: For monitoring
//! - LoggingInterface: For request logging

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use registrar::{
    Module, ModuleType, ModuleStatus,
    LocalRegistry, RegistryError,
};
use thiserror::Error;
use tower_http::trace::TraceLayer;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Registry(RegistryError::ModuleNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ApiError::Registry(RegistryError::ModuleExists(_)) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

type Result<T> = std::result::Result<T, ApiError>;

// API response types
#[derive(Debug, Serialize)]
struct ModuleResponse {
    name: String,
    status: ModuleStatus,
    #[serde(rename = "type")]
    module_type: ModuleType,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateModuleRequest {
    name: String,
    #[serde(rename = "type")]
    module_type: ModuleType,
}

// State type for sharing the registry across handlers
#[derive(Clone)]
struct AppState {
    registry: Arc<LocalRegistry>,
}

// API handlers
async fn list_modules(State(state): State<AppState>) -> Result<Json<Vec<ModuleResponse>>> {
    let modules = state.registry.list_modules().await?;
    let response = modules
        .into_iter()
        .map(|m| ModuleResponse {
            name: m.name,
            status: m.status,
            module_type: m.module_type,
        })
        .collect();
    Ok(Json(response))
}

async fn get_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ModuleResponse>> {
    let module = state.registry.get_module(&name).await?;
    Ok(Json(ModuleResponse {
        name: module.name,
        status: module.status,
        module_type: module.module_type,
    }))
}

async fn create_module(
    State(state): State<AppState>,
    Json(request): Json<CreateModuleRequest>,
) -> Result<StatusCode> {
    let module = Module {
        name: request.name.clone(),
        module_type: request.module_type,
        status: ModuleStatus::new(),
    };
    state.registry.register_module(module).await?;
    Ok(StatusCode::CREATED)
}

async fn delete_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.registry.unregister_module(&name).await?;
    Ok(StatusCode::OK)
}

async fn get_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ModuleStatus>> {
    let status = state.registry.get_module_status(&name).await?;
    Ok(Json(status))
}

async fn update_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(status): Json<ModuleStatus>,
) -> Result<StatusCode> {
    let mut module = state.registry.get_module(&name).await?;
    module.status = status;
    state.registry.register_module(module).await?;
    Ok(StatusCode::OK)
}

async fn start_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    let module = state.registry.get_module(&name).await?;
    state.registry.start_module(&name).await?;
    Ok(StatusCode::OK)
}

pub fn create_router(registry: LocalRegistry) -> Router {
    let state = AppState {
        registry: Arc::new(registry),
    };

    Router::new()
        .route("/modules", get(list_modules).post(create_module))
        .route(
            "/modules/:name",
            get(get_module).delete(delete_module),
        )
        .route(
            "/modules/:name/status",
            get(get_module_status).put(update_module_status),
        )
        .route("/modules/:name/start", axum::routing::post(start_module))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub mod client;
pub use client::{RegistrarClient, ClientError};

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::path::PathBuf;
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/test_registry"));
        create_router(registry)
    }

    #[tokio::test]
    async fn test_list_modules() {
        let app = setup_test_app().await;
        
        let response = app
            .oneshot(Request::builder().uri("/modules").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_module() {
        let app = setup_test_app().await;
        
        let request = CreateModuleRequest {
            name: "test-module".to_string(),
            module_type: ModuleType::Python {
                module_path: PathBuf::from("test.py"),
                requirements_path: None,
                venv_path: None,
            },
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/modules")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
