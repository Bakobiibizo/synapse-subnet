//! Validator management components

use super::*;
use crate::interface::core::models::{ValidatorStatus, ValidatorConfig, ValidatorMetrics};

/// Validator dashboard component
#[derive(Template)]
#[template(path = "components/validator/dashboard.html")]
pub struct ValidatorDashboard {
    /// Overall validator status
    pub status: ValidatorStatus,
    /// Performance metrics
    pub metrics: ValidatorMetrics,
    /// Active modules
    pub active_modules: Vec<String>,
}

/// Module validation status component
#[derive(Template)]
#[template(path = "components/validator/module_status.html")]
pub struct ModuleValidationStatus {
    /// Module name
    pub name: String,
    /// Validation status
    pub status: ValidatorStatus,
    /// Module metrics
    pub metrics: ValidatorMetrics,
}

/// Validator configuration form
#[derive(Template)]
#[template(path = "components/validator/config_form.html")]
pub struct ValidatorConfigForm {
    /// Current configuration
    pub config: ValidatorConfig,
    /// Form errors
    pub errors: Vec<String>,
}

impl Component for ValidatorDashboard {}
impl Component for ModuleValidationStatus {}
impl Component for ValidatorConfigForm {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    /// Test dashboard rendering
    #[tokio::test]
    async fn test_dashboard_render() {
        let component = ValidatorDashboard {
            status: ValidatorStatus {
                is_active: true,
                uptime: 3600,
                last_update: chrono::Utc::now(),
            },
            metrics: ValidatorMetrics {
                total_validations: 100,
                success_rate: 0.95,
                average_response_time: 150,
            },
            active_modules: vec!["test_module".to_string()],
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that response contains key metrics
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("95%")); // Success rate
        assert!(html.contains("test_module")); // Active module
    }

    /// Test configuration form validation
    #[tokio::test]
    async fn test_config_form_validation() {
        let component = ValidatorConfigForm {
            config: ValidatorConfig {
                max_modules: 10,
                timeout_ms: 5000,
                auto_restart: true,
            },
            errors: vec!["Invalid timeout value".to_string()],
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that response contains error message
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Invalid timeout value"));
    }
}
