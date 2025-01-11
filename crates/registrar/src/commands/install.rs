use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use clap::Parser;
use anyhow::{Result, Context};
use tokio::process::Command as TokioCommand;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");
const MODULES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/subnet-modules");

#[derive(Parser, Debug)]
#[clap(about = "Install and start the registrar for a subnet module")]
pub struct InstallCommand {
    /// Name of the subnet module to install. This must be a module name that was previously ingested, not a URL.
    pub name: String,

    /// Skip environment setup and use existing environment. Use this if you've already configured the environment.
    #[clap(long)]
    pub skip_env_setup: bool,

    /// Skip Docker build. Use this if you've already built the Docker image.
    #[clap(long)]
    pub skip_docker_build: bool,

    /// Port for the registrar to run on. Make sure this port is available.
    #[clap(long, default_value = "8080")]
    pub registrar_port: u16,
}

impl InstallCommand {
    pub async fn run(&self) -> Result<()> {
        // Validate that the name is not a URL
        if self.name.starts_with("http://") || self.name.starts_with("https://") {
            anyhow::bail!("The name parameter should be a module name, not a URL. To install directly from a repository, use:\n\ncargo run -p registrar -- register --repo {}", self.name);
        }

        let module_config_dir = PathBuf::from(CONFIG_DIR).join(&self.name);
        let module_dir = PathBuf::from(MODULES_DIR).join(&self.name);

        // Verify module exists
        if !module_config_dir.exists() {
            anyhow::bail!("Module '{}' not found in config directory. Have you ingested it first? Try:\n\ncargo run -p registrar -- register --repo <repository-url>", self.name);
        }
        if !module_dir.exists() {
            anyhow::bail!("Module '{}' not found in modules directory", self.name);
        }

        // Check for Docker
        if !self.skip_docker_build && !is_docker_installed()? {
            anyhow::bail!("Docker is not installed. Please install Docker first.");
        }

        // Setup environment if needed
        if !self.skip_env_setup {
            self.setup_environment(&module_config_dir)?;
        }

        // Build Docker image
        if !self.skip_docker_build {
            println!("Building Docker image...");
            self.build_docker_image(&module_dir)?;
        }

        // Start registrar
        println!("Starting registrar...");
        let registrar_handle = self.start_registrar().await?;

        println!("\nRegistrar is running on port {}.", self.registrar_port);
        println!("You can now register validators with this registrar using the validator CLI:");
        println!("\ncargo run -p validator -- register --registrar-url http://localhost:{} --config {}/config.yaml", 
            self.registrar_port, 
            module_config_dir.display()
        );

        // Wait for Ctrl+C
        println!("\nPress Ctrl+C to stop the registrar...");
        tokio::signal::ctrl_c().await?;

        // Cleanup
        println!("Cleaning up...");
        if let Some(mut handle) = registrar_handle {
            handle.kill().await?;
        }

        Ok(())
    }

    fn setup_environment(&self, config_dir: &Path) -> Result<()> {
        let env_example = config_dir.join(".env.example");
        
        // Check if environment exists
        let exists = Command::new("cargo")
            .args([
                "run", "-p", "registrar", "--",
                "env", "list"
            ])
            .output()?
            .stdout
            .windows(self.name.len())
            .any(|window| window == self.name.as_bytes());

        if !exists {
            println!("Creating {} environment...", self.name);
            Command::new("cargo")
                .args([
                    "run", "-p", "registrar", "--",
                    "env", "create", &self.name,
                    "--from-example", &env_example.to_string_lossy(),
                    "--accept-defaults"
                ])
                .status()
                .context("Failed to create environment")?;
        }

        println!("Activating environment...");
        Command::new("cargo")
            .args([
                "run", "-p", "registrar", "--",
                "env", "activate", &self.name
            ])
            .status()
            .context("Failed to activate environment")?;

        Ok(())
    }

    fn build_docker_image(&self, module_dir: &Path) -> Result<()> {
        Command::new("docker")
            .args(["build", "-t", &format!("{}:latest", self.name), "."])
            .current_dir(module_dir)
            .status()
            .context("Failed to build Docker image")?;
        Ok(())
    }

    async fn start_registrar(&self) -> Result<Option<tokio::process::Child>> {
        let child = TokioCommand::new("cargo")
            .args([
                "run", "-p", "registrar", "--",
                "start",
                "--port", &self.registrar_port.to_string()
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start registrar")?;

        // Give it time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        Ok(Some(child))
    }
}

fn is_docker_installed() -> Result<bool> {
    let output = Command::new("docker")
        .arg("--version")
        .output()
        .context("Failed to check Docker installation")?;
    Ok(output.status.success())
}
