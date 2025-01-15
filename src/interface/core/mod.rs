//! Core functionality shared across interface components

pub mod auth;
pub mod db;
pub mod models;
pub mod error;

pub mod prelude {
    pub use super::auth::{KeyPair, KeySignature, AuthManager};
    pub use super::db::{Database, Migration};
    pub use super::models::*;
    pub use super::error::{Error, Result};
}
