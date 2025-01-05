use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use synapse_registrar::{
    LocalRegistry, ModuleConfig, ModuleStatus, ModuleType, RegistryError, Registry,
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
    registry: Arc<dyn Registry>,
}

// API handlers
async fn list_modules(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModuleResponse>>> {
    let modules = state.registry.list_modules().await?;
    let response = modules
        .into_iter()
        .map(|config| ModuleResponse {
            name: config.name,
            status: config.status,
            module_type: config.module_type,
        })
        .collect();
    Ok(Json(response))
}

async fn get_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ModuleResponse>> {
    let config = state.registry.get_module(&name).await?;
    Ok(Json(ModuleResponse {
        name: config.name,
        status: config.status,
        module_type: config.module_type,
    }))
}

async fn create_module(
    State(state): State<AppState>,
    Json(request): Json<CreateModuleRequest>,
) -> Result<StatusCode> {
    let config = ModuleConfig::new(request.name, request.module_type);
    state.registry.register_module(config).await?;
    Ok(StatusCode::CREATED)
}

async fn update_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<CreateModuleRequest>,
) -> Result<StatusCode> {
    let mut config = state.registry.get_module(&name).await?;
    config.name = request.name;
    config.module_type = request.module_type;
    state.registry.update_module(&name, config).await?;
    Ok(StatusCode::OK)
}

async fn delete_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.registry.remove_module(&name).await?;
    Ok(StatusCode::NO_CONTENT)
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
    let mut config = state.registry.get_module(&name).await?;
    config.status = status;
    state.registry.update_module(&name, config).await?;
    Ok(StatusCode::OK)
}

pub fn create_router(registry: impl Registry + 'static) -> Router {
    let state = AppState {
        registry: Arc::new(registry),
    };

    Router::new()
        .route("/modules", get(list_modules).post(create_module))
        .route(
            "/modules/:name",
            get(get_module)
                .put(update_module)
                .delete(delete_module),
        )
        .route(
            "/modules/:name/status",
            get(get_module_status).put(update_module_status),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

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
