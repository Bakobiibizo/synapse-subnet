//! Core functionality for the Synapse Interface system
//! 
//! This library provides shared functionality for CLI, API, and GUI interfaces
//! including authentication, database operations, and common types.

pub mod auth;
pub mod db;
pub mod models;
pub mod error;

pub use error::{Error, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use super::auth::{KeyPair, KeySignature};
    pub use super::models::*;
    pub use super::error::{Error, Result};
    pub use super::db::Database;
}
