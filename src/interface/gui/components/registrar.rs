//! Registrar management components

use super::*;
use crate::interface::core::models::Module;

/// Module list component
#[derive(Template)]
#[template(path = "components/registrar/module_list.html")]
pub struct ModuleList {
    /// List of modules
    pub modules: Vec<Module>,
    /// Current filter
    pub filter: Option<String>,
}

/// Module registration form
#[derive(Template)]
#[template(path = "components/registrar/module_form.html")]
pub struct ModuleForm {
    /// Form action URL
    pub action: String,
    /// Optional existing module
    pub module: Option<Module>,
    /// Form errors
    pub errors: Vec<String>,
}

/// Module status component
#[derive(Template)]
#[template(path = "components/registrar/module_status.html")]
pub struct ModuleStatus {
    /// Module name
    pub name: String,
    /// Current status
    pub status: crate::interface::core::models::ModuleStatus,
    /// Update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Component for ModuleList {}
impl Component for ModuleForm {}
impl Component for ModuleStatus {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    /// Test module list rendering
    #[tokio::test]
    async fn test_module_list_render() {
        let modules = vec![
            Module {
                name: "test_module".to_string(),
                version: "1.0.0".to_string(),
                // ... other fields
            }
        ];

        let component = ModuleList {
            modules,
            filter: None,
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that the response contains the module name
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("test_module"));
    }

    /// Test module form validation
    #[tokio::test]
    async fn test_module_form_validation() {
        let component = ModuleForm {
            action: "/api/registrar/modules".to_string(),
            module: None,
            errors: vec!["Name is required".to_string()],
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that the response contains the error message
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Name is required"));
    }
}
