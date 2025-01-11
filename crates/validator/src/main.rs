use std::sync::Arc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use docker_manager::DockerManager;
use registrar_api::client::{RegistrarClient, RegistrarClientTrait};
use validator::{ValidatorManager, api};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register this validator with a registrar
    Register {
        /// URL of the registrar to register with
        #[arg(long)]
        registrar_url: String,
    },
    /// Start the validator
    Start {
        /// Port to run the validator on
        #[arg(long, default_value = "8081")]
        port: u16,

        /// Optional registrar URL to connect to
        #[arg(long)]
        registrar_url: Option<String>,

        /// Path to config file
        #[arg(long)]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Register { registrar_url } => {
            println!("Registering with registrar at {}", registrar_url);
            
            // Create registrar client and register
            let registrar = RegistrarClient::new(registrar_url);
            registrar.register().await?;
            
            println!("Successfully registered with registrar");
            Ok(())
        }
        Commands::Start { port, registrar_url, config } => {
            println!("Starting validator on port {}", port);
            if let Some(url) = &registrar_url {
                println!("Connected to registrar at {}", url);
            }
            println!("Using config file: {}", config);

            // Create Docker manager
            let docker = Arc::new(DockerManager::new().await?);

            // Create validator manager
            let validator = if let Some(url) = registrar_url {
                let registrar = Box::new(RegistrarClient::new(url));
                Arc::new(ValidatorManager::new(docker, registrar))
            } else {
                // Create validator without registrar connection
                Arc::new(ValidatorManager::new(docker, Box::new(NoopRegistrarClient)))
            };

            // Create API router
            let app = api::router(validator);

            // Start API server
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            println!("Starting validator API server on http://{}", addr);
            axum::serve(
                tokio::net::TcpListener::bind(addr).await?,
                app
            ).await?;
            
            Ok(())
        }
    }
}

// No-op registrar client for when we don't want to connect to a registrar
struct NoopRegistrarClient;

#[async_trait::async_trait]
impl RegistrarClientTrait for NoopRegistrarClient {
    async fn register(&self) -> Result<(), registrar_api::client::ClientError> {
        Ok(())
    }

    async fn get_config(&self, _module: &str) -> Result<String, registrar_api::client::ClientError> {
        Ok("".to_string())
    }

    async fn get_env_template(&self, _module: &str) -> Result<String, registrar_api::client::ClientError> {
        Ok("".to_string())
    }
}
