//! API request handlers

pub mod registrar;
pub mod validator;
pub mod miner;
pub mod auth;

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::interface::core::{auth::KeySignature, error::Result};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Database connection
    pub db: crate::interface::core::db::Database,
    /// Environment manager
    pub env_manager: std::sync::Arc<tokio::sync::Mutex<crate::interface::core::environment::EnvironmentManager>>,
}

/// Common response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: Option<T>,
    /// Error message if any
    pub error: Option<String>,
    /// Response metadata
    pub meta: ResponseMeta,
}

/// Response metadata
#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    /// Request timestamp
    pub timestamp: i64,
    /// Request ID
    pub request_id: String,
}

/// Authentication request
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    /// Message to verify
    pub message: String,
    /// Signature details
    pub signature: KeySignature,
}
