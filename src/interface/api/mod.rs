//! REST API Server with WebSocket support
//! 
//! Provides HTTP endpoints for subnet interaction and real-time updates

use axum::{
    routing::{get, post},
    Router, Extension,
};
use tower_http::cors::CorsLayer;
use crate::interface::core::prelude::*;

mod handlers;
mod websocket;
mod swagger;

pub struct ApiServer {
    db: Database,
    auth: AuthManager,
}

impl ApiServer {
    pub async fn new(db: Database) -> Result<Self, Box<dyn std::error::Error>> {
        let auth = AuthManager::new(db.clone()).await?;
        Ok(Self { db, auth })
    }

    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            // REST endpoints
            .route("/api/registrar/*path", get(handlers::registrar::handle_get).post(handlers::registrar::handle_post))
            .route("/api/validator/*path", get(handlers::validator::handle_get).post(handlers::validator::handle_post))
            .route("/api/miner/*path", get(handlers::miner::handle_get).post(handlers::miner::handle_post))
            // WebSocket endpoint
            .route("/ws", get(websocket::handler))
            // Swagger docs
            .route("/docs/openapi.json", get(swagger::openapi_json))
            .route("/docs", get(swagger::swagger_ui))
            // Middleware
            .layer(CorsLayer::permissive())
            .layer(Extension(self.db))
            .layer(Extension(self.auth));

        axum::Server::bind(&addr.parse()?)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}
