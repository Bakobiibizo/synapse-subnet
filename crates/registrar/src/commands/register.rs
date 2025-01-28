use clap::Parser;
use anyhow::Result;
use crate::commands::{ingest::IngestCommand, install::InstallCommand};

#[derive(Parser, Debug)]
#[clap(about = "Register a subnet module from a repository")]
pub struct RegisterCommand {
    /// URL of the repository to register
    #[clap(long)]
    pub repo: String,

    /// Port for the registrar to run on
    #[clap(long, default_value = "8080")]
    pub registrar_port: u16,

    /// Accept all defaults for environment variables
    #[clap(long)]
    pub accept_defaults: bool,

    /// Skip Docker build
    #[clap(long)]
    pub skip_docker_build: bool,

    /// Environment variables to set (format: KEY=VALUE)
    #[clap(long = "env", value_parser = parse_key_val)]
    pub env_vars: Vec<(String, String)>,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

impl RegisterCommand {
    pub async fn run(&self) -> Result<()> {
        // First ingest the module
        let ingest_cmd = IngestCommand {
            repo: self.repo.clone(),
            name: None,
            branch: "main".to_string(),
        };
        
        println!("Attempting to clone repository: {}", self.repo);
        let module_name = ingest_cmd.run().await?;
        println!("Successfully cloned module: {}", module_name);

        // Create environment file
        let config_dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/config"));
        let env_file = config_dir.join(&module_name).join(".env");
        let mut env_content = String::new();
        for (key, value) in &self.env_vars {
            env_content.push_str(&format!("{}={}\n", key, value));
        }
        std::fs::write(&env_file, env_content)?;

        // Then install it
        let install_cmd = InstallCommand {
            name: module_name,
            skip_env_setup: !self.env_vars.is_empty(), // Skip if we set env vars
            skip_docker_build: self.skip_docker_build,
            registrar_port: self.registrar_port,
        };
        install_cmd.run().await?;

        Ok(())
    }
}
