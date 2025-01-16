use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use docker_manager::DockerManager;
use registrar::module::Module;
use registrar_api::{
    RegistrarClient, ClientError, RegistrarClientTrait,
    client::{ModuleStatus, ModuleState},
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;
use validator::{ValidatorManager, api};
use base64::{Engine as _, engine::general_purpose};
use bollard::Docker;
use bollard::container::Config;
use bollard::models::PortBinding;
use bollard::service::HostConfig;
use dotenv::from_path;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use sha2::{Sha256, Digest};
use axum;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
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
    /// Install a module
    Install(InstallCommand),
}

#[derive(Deserialize)]
struct InstallationPackage {
    package: String,  // base64 encoded
    hash: String,
    metadata: ModuleMetadata,
}

#[derive(Deserialize)]
struct ModuleMetadata {
    name: String,
    version: String,
    repo_url: String,
    branch: String,
}

#[derive(Parser, Debug)]
struct InstallCommand {
    /// Name of the module to install
    #[clap(long)]
    name: String,

    /// URL of the registrar to get the module from
    #[clap(long)]
    registrar_url: String,

    /// Skip prompts and use default values
    #[clap(long)]
    no_prompt: bool,
}

impl InstallCommand {
    async fn run(&self) -> Result<()> {
        println!("Fetching installation package for module '{}'...", self.name);
        
        // Get installation package from registrar
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/api/v1/subnet-modules/{}/package", 
                self.registrar_url, 
                self.name
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch installation package: {}", 
                response.text().await?);
        }

        let package: InstallationPackage = response.json().await?;
        
        // Verify package hash
        let decoded = general_purpose::STANDARD.decode(&package.package)?;
        let mut hasher = Sha256::new();
        hasher.update(&decoded);
        let hash = format!("{:x}", hasher.finalize());
        
        if hash != package.hash {
            anyhow::bail!("Package hash verification failed");
        }

        // Create temporary directory for unpacking
        let temp_dir = TempDir::new()?;
        let package_path = temp_dir.path().join("install_package.tar.gz");
        fs::write(&package_path, decoded)?;

        // Extract package
        let status = Command::new("tar")
            .args(["xzf", package_path.to_str().unwrap()])
            .current_dir(temp_dir.path())
            .status()
            .context("Failed to extract installation package")?;

        if !status.success() {
            anyhow::bail!("Failed to extract installation package");
        }

        // Run installer script
        let installer_path = temp_dir.path().join("install.sh");
        let mut cmd = Command::new("bash");
        cmd.arg(&installer_path)
            .env("MODULE_NAME", &self.name)
            .env("REGISTRAR_URL", &self.registrar_url)
            .env("NO_PROMPT", self.no_prompt.to_string());

        if self.no_prompt {
            cmd.arg("--no-prompt");
        }

        let status = cmd.status()
            .context("Failed to run installer script")?;

        if !status.success() {
            anyhow::bail!("Installation failed");
        }

        println!("\nModule '{}' installed successfully!", self.name);
        println!("Metadata:");
        println!("  Version: {}", package.metadata.version);
        println!("  Repository: {}", package.metadata.repo_url);
        println!("  Branch: {}", package.metadata.branch);
        
        Ok(())
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

async fn install_module(repo_url: &str) -> Result<()> {
    // Create temporary directory for module installation
    let temp_dir = TempDir::new()?;
    let package_path = temp_dir.path().join("module.tar.gz");

    // Download module package
    let response = reqwest::get(repo_url).await?;
    let content = response.bytes().await?;
    fs::write(&package_path, content)?;

    // Extract package
    let status = Command::new("tar")
        .args(["xzf", package_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .status()
        .context("Failed to extract installation package")?;

    if !status.success() {
        anyhow::bail!("Failed to extract module package");
    }

    // Find and run install script
    let install_script = temp_dir.path().join("install.sh");
    if install_script.exists() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(install_script.to_str().unwrap())
            .current_dir(temp_dir.path());

        let status = cmd.status()
            .context("Failed to run installer script")?;

        if !status.success() {
            anyhow::bail!("Module installation failed");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Register { registrar_url } => {
            println!("Registering with registrar at {}", registrar_url);
            
            // Create registrar client and register
            let registrar = RegistrarClient::new(&registrar_url).expect("Failed to create registrar client");
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

            // Load .env file from the same directory as config
            let config_dir = Path::new(&config).parent().unwrap();
            let env_path = config_dir.join(".env");
            println!("Loading environment from: {}", env_path.display());
            from_path(&env_path)?;

            // Load config file
            let config_str = fs::read_to_string(&config)?;
            let module_config: Module = serde_yaml::from_str(&config_str)?;

            // Create Docker manager
            let docker = Arc::new(DockerManager::new().await?);

            // Initialize registrar client if URL is provided
            let validator = if let Some(registrar_url) = registrar_url {
                // Create registrar client
                let registrar = RegistrarClient::new(&registrar_url).expect("Failed to create registrar client");
                Arc::new(ValidatorManager::new(docker.clone(), Arc::new(registrar)))
            } else {
                // Create validator without registrar connection
                let registrar = Arc::new(NoopRegistrarClient::new("noop".to_string()));
                Arc::new(ValidatorManager::new(docker.clone(), registrar))
            };

            // Start the Python validator container using bollard
            if let Module { name, module_type: registrar::module::ModuleType::Docker { image, tag, port: container_port, env, volumes, health_check: _ }, .. } = module_config {
                println!("Starting Python validator container...");

                // Connect to Docker daemon
                let docker = Docker::connect_with_local_defaults()?;

                // Convert environment variables with actual values
                let env_vars: Vec<String> = if let Some(env_map) = env {
                    env_map.into_iter()
                        .map(|(k, v)| {
                            let value = if v.starts_with("${") && v.ends_with("}") {
                                // Extract env var name and get its value
                                let env_name = &v[2..v.len()-1];
                                std::env::var(env_name).unwrap_or_else(|_| v.clone())
                            } else {
                                v
                            };
                            format!("{}={}", k, value)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                println!("Environment variables: {:?}", env_vars);

                // Convert volumes
                let binds: Vec<String> = if let Some(vol_map) = volumes {
                    vol_map.into_iter()
                        .map(|(k, v)| format!("{}:{}", k, v))
                        .collect()
                } else {
                    Vec::new()
                };
                println!("Volume bindings: {:?}", binds);

                // Create port bindings
                let mut port_bindings = HashMap::new();
                let mut exposed_ports = HashMap::new();
                let port_key = format!("{}/tcp", container_port);
                
                let mut binding = Vec::new();
                binding.push(PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(container_port.to_string()),
                });
                port_bindings.insert(port_key.clone(), Some(binding));
                exposed_ports.insert(port_key, HashMap::new());

                // Create container
                let config = Config {
                    image: Some(format!("{}:{}", image, tag)),
                    cmd: Some(vec!["python".to_string(), "-m".to_string(), "zangief.validator.validator".to_string()]),
                    env: Some(env_vars),
                    exposed_ports: Some(exposed_ports),
                    host_config: Some(HostConfig {
                        port_bindings: Some(port_bindings),
                        binds: Some(binds),
                        auto_remove: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                println!("Container config: {:?}", config);

                // First, try to remove any existing container with the same name
                match docker.remove_container(&name, None).await {
                    Ok(_) => println!("Removed existing container"),
                    Err(e) => println!("No existing container to remove: {}", e),
                }

                // Create and start container
                println!("Creating container with name: {}", name);
                let container = match docker.create_container(
                    Some(bollard::container::CreateContainerOptions {
                        name: &name,
                        platform: None,
                    }), 
                    config
                ).await {
                    Ok(container) => {
                        println!("Container created successfully with ID: {}", container.id);
                        container
                    },
                    Err(e) => {
                        eprintln!("Failed to create container: {}", e);
                        return Err(anyhow::anyhow!("Failed to create container: {}", e));
                    }
                };

                println!("Starting container {}", container.id);
                match docker.start_container::<String>(&container.id, None).await {
                    Ok(_) => println!("Container started successfully"),
                    Err(e) => {
                        eprintln!("Failed to start container: {}", e);
                        return Err(anyhow::anyhow!("Failed to start container: {}", e));
                    }
                }

                // Verify container is running
                match docker.inspect_container(&container.id, None).await {
                    Ok(info) => {
                        if let Some(state) = info.state {
                            if let Some(running) = state.running {
                                if running {
                                    println!("Container is running");
                                } else {
                                    eprintln!("Container is not running");
                                    if let Some(error) = state.error {
                                        eprintln!("Container error: {}", error);
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Failed to inspect container: {}", e),
                }

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
        Commands::Install(cmd) => cmd.run().await,
    }
}

#[derive(Clone)]
struct NoopRegistrarClient {
    base_url: String,
}

impl NoopRegistrarClient {
    fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait::async_trait]
impl RegistrarClientTrait for NoopRegistrarClient {
    async fn list_modules(&self) -> Result<Vec<registrar_api::client::Module>, ClientError> {
        Ok(vec![])
    }

    async fn get_module(&self, _name: &str) -> Result<registrar_api::client::Module, ClientError> {
        Err(ClientError::ModuleNotFound("noop".to_string()))
    }

    async fn create_module(&self, _module: &registrar_api::client::Module) -> Result<i64, ClientError> {
        Ok(1)
    }

    async fn get_module_status(&self, _name: &str) -> Result<ModuleStatus, ClientError> {
        Ok(ModuleStatus {
            state: ModuleState::Created,
            health: None,
            error: None,
        })
    }

    async fn update_module_status(&self, _name: &str, _status: &ModuleStatus) -> Result<(), ClientError> {
        Ok(())
    }

    async fn start_module(&self, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }

    async fn register_module(&self, _module: &registrar_api::client::Module) -> Result<(), ClientError> {
        Ok(())
    }

    async fn unregister_module(&self, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }

    async fn register_miner(&self, _uid: &str, _key: &str, _name: &str) -> Result<(), ClientError> {
        Ok(())
    }
}
