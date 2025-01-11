use axum::Router;
use clap::Parser;
use anyhow::Result;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tokio::net::TcpListener;

mod api {
    use std::path::PathBuf;
    use axum::{
        extract::{Path as AxumPath},
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use tokio::fs;
    use tower_http::services::ServeDir;

    const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

    pub fn router() -> Router {
        Router::new()
            .route("/modules/:name/config", get(get_config))
            .route("/modules/:name/env-template", get(get_env_template))
            .nest_service("/v2", ServeDir::new(CONFIG_DIR))
    }

    async fn get_config(
        AxumPath(name): AxumPath<String>,
    ) -> impl IntoResponse {
        let config_path = PathBuf::from(CONFIG_DIR)
            .join(&name)
            .join("config.yaml");
        
        match fs::read_to_string(config_path).await {
            Ok(content) => Response::builder()
                .header("content-type", "application/yaml")
                .body(content)
                .unwrap(),
            Err(_) => Response::builder()
                .status(404)
                .body(format!("Module {} not found", name))
                .unwrap(),
        }
    }

    async fn get_env_template(
        AxumPath(name): AxumPath<String>,
    ) -> impl IntoResponse {
        let env_path = PathBuf::from(CONFIG_DIR)
            .join(&name)
            .join(".env.example");
        
        match fs::read_to_string(env_path).await {
            Ok(content) => Response::builder()
                .header("content-type", "text/plain")
                .body(content)
                .unwrap(),
            Err(_) => Response::builder()
                .status(404)
                .body(format!("Environment template for module {} not found", name))
                .unwrap(),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(about = "Start the registrar server")]
pub struct StartCommand {
    /// Port to run the server on. Make sure this port is available and not used by another service.
    #[clap(long, default_value = "8080")]
    pub port: u16,
}

impl StartCommand {
    pub async fn run(&self) -> Result<()> {
        let app = Router::new()
            .merge(api::router())
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        println!("Registrar listening on http://{}", addr);

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
