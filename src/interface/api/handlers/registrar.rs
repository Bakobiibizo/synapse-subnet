//! Registrar API handlers

use super::*;
use axum::extract::Path;

/// Register a new module
///
/// POST /api/registrar/modules
pub async fn register_module(
    State(state): State<AppState>,
    Json(module): Json<crate::interface::core::models::Module>,
) -> impl IntoResponse {
    todo!("Implement module registration")
}

/// Unregister a module
///
/// DELETE /api/registrar/modules/{name}
pub async fn unregister_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    todo!("Implement module unregistration")
}

/// List all modules
///
/// GET /api/registrar/modules
pub async fn list_modules(
    State(state): State<AppState>,
) -> impl IntoResponse {
    todo!("Implement module listing")
}

/// Get module status
///
/// GET /api/registrar/modules/{name}/status
pub async fn get_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    todo!("Implement status retrieval")
}

/// Update module status
///
/// PUT /api/registrar/modules/{name}/status
pub async fn update_module_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(status): Json<crate::interface::core::models::ModuleStatus>,
) -> impl IntoResponse {
    todo!("Implement status update")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    /// Test module registration
    #[tokio::test]
    async fn test_register_module() {
        let app = create_test_app().await;
        let module = crate::interface::core::models::Module {
            name: "test_module".to_string(),
            version: "1.0.0".to_string(),
            // ... other fields
        };

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/registrar/modules")
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&module).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// Test module listing
    #[tokio::test]
    async fn test_list_modules() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/api/registrar/modules")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Create test application
    async fn create_test_app() -> axum::Router {
        // Create test state
        let state = AppState {
            db: crate::interface::core::db::Database::connect("sqlite::memory:").await.unwrap(),
            env_manager: std::sync::Arc::new(tokio::sync::Mutex::new(
                crate::interface::core::environment::EnvironmentManager::new(
                    std::path::PathBuf::from("/tmp/test"),
                    std::sync::Arc::new(crate::interface::core::environment::tests::MockDockerManager::default()),
                )
                .await
                .unwrap(),
            )),
        };

        // Create router with test state
        axum::Router::new()
            .route("/api/registrar/modules", axum::routing::post(register_module))
            .route("/api/registrar/modules", axum::routing::get(list_modules))
            .with_state(state)
    }
}
