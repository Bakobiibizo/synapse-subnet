//! HTMX-based GUI components

pub mod registrar;
pub mod validator;
pub mod miner;
pub mod layout;

use askama::Template;
use axum::{
    response::{IntoResponse, Response},
    http::HeaderMap,
};

/// Base trait for all components
pub trait Component: Template {
    /// Convert component to HTTP response
    fn into_response(self) -> Response where Self: Sized {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "text/html".parse().unwrap());
        
        match self.render() {
            Ok(html) => (headers, html).into_response(),
            Err(err) => (
                headers,
                format!("Template Error: {}", err)
            ).into_response(),
        }
    }
}

/// Component state
#[derive(Clone)]
pub struct ComponentState {
    /// Database connection
    pub db: crate::interface::core::db::Database,
    /// Environment manager
    pub env_manager: std::sync::Arc<tokio::sync::Mutex<crate::interface::core::environment::EnvironmentManager>>,
}
