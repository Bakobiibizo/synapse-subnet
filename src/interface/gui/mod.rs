//! Web GUI using HTMX components
//! 
//! Provides a lightweight web interface for human interaction with the subnet

use axum::{
    routing::get,
    Router, Extension,
};
use askama::Template;
use crate::interface::core::prelude::*;

mod components;
mod templates;
mod state;

pub struct GuiServer {
    db: Database,
    auth: AuthManager,
    state: state::GuiState,
}

impl GuiServer {
    pub async fn new(db: Database) -> Result<Self, Box<dyn std::error::Error>> {
        let auth = AuthManager::new(db.clone()).await?;
        let state = state::GuiState::new();
        Ok(Self { db, auth, state })
    }

    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            // Main routes
            .route("/", get(templates::index::handler))
            .route("/dashboard", get(templates::dashboard::handler))
            // Component routes
            .route("/components/registrar/*path", get(components::registrar::handler))
            .route("/components/validator/*path", get(components::validator::handler))
            .route("/components/miner/*path", get(components::miner::handler))
            // Static files
            .route("/static/*path", get(templates::static_handler))
            // State management
            .layer(Extension(self.state))
            .layer(Extension(self.db))
            .layer(Extension(self.auth));

        axum::Server::bind(&addr.parse()?)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
