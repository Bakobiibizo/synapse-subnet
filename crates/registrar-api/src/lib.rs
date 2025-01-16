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
use registrar_core::{
    Module, ModuleType, ModuleStatus,
    Registry, RegistryError,
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

// API response types
#[derive(Debug, Serialize)]
pub struct ModuleResponse {
    name: String,
    status: ModuleStatus,
    #[serde(rename = "type")]
    module_type: ModuleType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    name: String,
    #[serde(rename = "type")]
    module_type: ModuleType,
}

// State type for sharing the registry across handlers
#[derive(Clone)]
pub struct AppState {
    registry: Arc<dyn Registry>,
}

// API handlers
async fn list_modules(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<ModuleResponse>>, ApiError> {
    let modules = state.registry.list_modules().await?;
    Ok(Json(
        modules
            .into_iter()
            .map(|m| ModuleResponse {
                name: m.name,
                status: m.status,
                module_type: m.module_type,
            })
            .collect(),
    ))
}

async fn get_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> std::result::Result<Json<ModuleResponse>, ApiError> {
    let module = state.registry.get_module(&name).await?;
    match module {
        Some(m) => Ok(Json(ModuleResponse {
            name: m.name,
            status: m.status,
            module_type: m.module_type,
        })),
        None => Err(ApiError::Registry(RegistryError::ModuleNotFound(name))),
    }
}

async fn create_module(
    State(state): State<AppState>,
    Json(request): Json<CreateModuleRequest>,
) -> std::result::Result<StatusCode, ApiError> {
    let module = Module {
        name: request.name,
        module_type: request.module_type,
        status: ModuleStatus::Stopped,
    };
    state.registry.create_module(&module).await?;
    Ok(StatusCode::CREATED)
}

async fn delete_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, ApiError> {
    state.registry.unregister_module(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> std::result::Result<Json<ModuleStatus>, ApiError> {
    let module = state.registry.get_module(&name).await?;
    match module {
        Some(m) => Ok(Json(m.status)),
        None => Err(ApiError::Registry(RegistryError::ModuleNotFound(name))),
    }
}

async fn update_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(status): Json<ModuleStatus>,
) -> std::result::Result<StatusCode, ApiError> {
    state.registry.update_module_status(&name, status).await?;
    Ok(StatusCode::OK)
}

async fn start_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> std::result::Result<StatusCode, ApiError> {
    let module = state.registry.get_module(&name).await?;
    match module {
        Some(_) => {
            state.registry.start_module(&name).await?;
            Ok(StatusCode::OK)
        }
        None => Err(ApiError::Registry(RegistryError::ModuleNotFound(name))),
    }
}

pub fn create_router(registry: impl Registry + 'static) -> Router {
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
pub mod traits;

pub use client::{RegistrarClient, ClientError, ModuleState};
pub use traits::RegistrarClientTrait;

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
