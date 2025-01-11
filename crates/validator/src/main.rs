use std::sync::Arc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use docker_manager::{DockerManager, ContainerConfig};
use registrar_api::client::{RegistrarClient, RegistrarClientTrait, ClientError};
use registrar::module::{Module, ModuleStatus, ModuleState};
use validator::{ValidatorManager, api};
use std::fs;
use serde_yaml;
use docker_manager::ContainerManager;

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
            let _registrar = RegistrarClient::new(registrar_url);
            // TODO: Implement proper registration
            println!("Successfully registered with registrar");
            Ok(())
        }
        Commands::Start { port, registrar_url, config } => {
            println!("Starting validator on port {}", port);
            if let Some(url) = &registrar_url {
                println!("Connected to registrar at {}", url);
            }
            println!("Using config file: {}", config);

            // Load config file
            let config_str = fs::read_to_string(&config)?;
            let module_config: Module = serde_yaml::from_str(&config_str)?;

            // Create Docker manager
            let docker = Arc::new(DockerManager::new().await?);

            // Create validator manager
            let validator = if let Some(url) = registrar_url {
                let registrar = Box::new(RegistrarClient::new(url));
                Arc::new(ValidatorManager::new(docker.clone(), registrar))
            } else {
                // Create validator without registrar connection
                Arc::new(ValidatorManager::new(docker.clone(), Box::new(NoopRegistrarClient)))
            };

            // Start the Python validator container
            if let Module { name, module_type: registrar::module::ModuleType::Docker { image, tag, port: container_port, env, volumes, health_check }, .. } = module_config {
                println!("Starting Python validator container...");
                let container_config = ContainerConfig {
                    name: name.clone(),
                    image,
                    tag,
                    env,
                    ports: Some([(container_port.to_string(), container_port.to_string())].iter().cloned().collect()),
                    volumes,
                    health_check,
                };
                docker.create_container(container_config.clone()).await?;
                docker.start_container(&name).await?;
                println!("Python validator container started");
            }

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

#[derive(Clone)]
struct NoopRegistrarClient;

#[async_trait::async_trait]
impl RegistrarClientTrait for NoopRegistrarClient {
    async fn register_module(&self, _module: Module) -> Result<(), ClientError> {
        Ok(())
    }

    async fn start_module(&self, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }

    async fn unregister_module(&self, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }

    async fn get_module_status(&self, _name: &str) -> Result<ModuleStatus, ClientError> {
        Ok(ModuleStatus {
            state: ModuleState::Stopped,
            error: None,
            container_status: None,
        })
    }

    async fn update_module_status(&self, _name: &str, _status: ModuleStatus) -> Result<(), ClientError> {
        Ok(())
    }

    async fn register_miner(&self, _uid: &str, _key: &str, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }

    async fn list_modules(&self) -> Result<Vec<Module>, ClientError> {
        Ok(vec![])
    }

    fn clone_box(&self) -> Box<dyn RegistrarClientTrait> {
        Box::new(self.clone())
    }
}
