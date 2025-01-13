use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use registrar_core::{ModuleType, ModuleStatus};
use time::OffsetDateTime;
use std::io::Write;
use crate::Registry;
use tokio::process::Command;
use sqlx;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");
const MODULES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/subnet-modules");

#[derive(Parser, Debug)]
#[clap(about = "Ingest a subnet module from a git repository")]
pub struct IngestCommand {
    /// Git repository URL to ingest. The repository should contain a Dockerfile and optionally an .env.example file.
    pub repo: String,

    /// Custom name for the module (defaults to repository name)
    #[clap(long)]
    pub name: Option<String>,

    /// Branch or tag to checkout (defaults to main)
    #[clap(long, default_value = "main")]
    pub branch: String,
}

impl IngestCommand {
    pub async fn run(&self) -> Result<String> {
        // Create modules directory if it doesn't exist
        std::fs::create_dir_all(MODULES_DIR)?;
        
        // Create temporary directory for cloning
        let temp_dir = tempfile::tempdir()?;
        
        // Clone the repository using git command
        println!("Cloning repository {}...", self.repo);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth", "1",
                "--branch", &self.branch,
                &self.repo,
                temp_dir.path().to_str().unwrap()
            ])
            .status()
            .await?;
            
        if !status.success() {
            anyhow::bail!("Failed to clone repository");
        }
        
        // Get module name from repo URL or custom name
        let name = self.name.clone().unwrap_or_else(|| {
            self.repo
                .split('/')
                .last()
                .unwrap()
                .trim_end_matches(".git")
                .to_string()
        });

        // Clean up existing module if it exists
        let db_url = format!("sqlite:{}/data/registrar.db", env!("CARGO_MANIFEST_DIR"));
        let registry = Registry::new(&db_url, PathBuf::from(CONFIG_DIR).join(&name)).await?;
        
        // Delete from database if exists
        sqlx::query!(
            r#"
            DELETE FROM subnet_modules
            WHERE name = ?1
            "#,
            name
        )
        .execute(&registry.db().clone())
        .await?;
        
        // Remove module directory
        let module_dir = PathBuf::from(CONFIG_DIR).join(&name);
        if module_dir.exists() {
            std::fs::remove_dir_all(&module_dir)?;
        }

        // Create module config directory
        let module_config_dir = PathBuf::from(CONFIG_DIR).join(&name);
        std::fs::create_dir_all(&module_config_dir)?;

        // Copy directory contents
        copy_dir_all(temp_dir.path(), &module_config_dir)?;

        // Read .env.example and prompt for values
        let temp_path = temp_dir.path();
        let env_example = if temp_path.join(".env.example").exists() {
            std::fs::read_to_string(temp_path.join(".env.example"))?
        } else {
            create_default_env_template(&name)
        };

        // Parse .env.example and prompt for values
        let mut env_content = String::new();
        
        // Always add GITHUB_REPOSITORY first
        env_content.push_str("GITHUB_REPOSITORY=hydra-dynamix/zangief\n");
        
        // Process each line from .env.example
        for line in env_example.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            // Skip if it's GITHUB_REPOSITORY since we already added it
            if line.contains("GITHUB_REPOSITORY") {
                continue;
            }

            if let Some((key, default_value)) = line.split_once('=') {
                let key = key.trim();
                let default_value = default_value.trim();

                print!("{} [default: {}]: ", key, default_value);
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let value = input.trim();

                let final_value = if value.is_empty() { default_value } else { value };
                env_content.push_str(&format!("{}={}\n", key, final_value));
            }
        }

        // Write the .env file
        std::fs::write(module_config_dir.join(".env"), env_content)?;

        // Create config.yaml
        let config = create_config_yaml(&name)?;
        std::fs::write(module_config_dir.join("config.yaml"), config)?;

        let registry_module = crate::registry::RegistryModule {
            id: 0,
            name: name.to_string(),
            version: "0.1.0".to_string(),
            repo_url: self.repo.to_string(),
            branch: self.branch.to_string(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            downloads: 0,
            module_type: ModuleType::Validator.to_string(),
            status: ModuleStatus::Stopped.to_string(),
        };

        registry.create_module(&registry_module).await?;

        println!("\nModule '{}' ingested successfully!", name);
        println!("Configuration files created in: {}", module_config_dir.display());
        
        // Launch with docker-compose
        println!("\nStarting module with docker-compose...");
        let docker_compose_path = format!("{}/{}/docker-compose.yml", CONFIG_DIR, name);
        let status = Command::new("docker")
            .args(["compose", "-f", &docker_compose_path, "up", "-d"])
            .status()
            .await?;

        if status.success() {
            println!("Module started successfully!");
        } else {
            println!("Warning: Failed to start module container");
        }

        Ok(name)
    }
}

fn create_default_env_template(name: &str) -> String {
    format!(
        r#"# {} Environment Variables
# Customize these values for your deployment

# Network Configuration
NETWORK_PORT=8000
NETWORK_HOST=0.0.0.0

# Authentication
API_KEY=your_api_key_here

# Module Settings
LOG_LEVEL=info
BATCH_SIZE=32
"#,
        name
    )
}

fn create_config_yaml(name: &str) -> Result<String> {
    Ok(format!(
        r#"# {} Configuration

# Network settings
network:
  port: 8000
  host: "0.0.0.0"

# Module settings
module:
  name: "{}"
  version: "0.1.0"
  log_level: "info"
  batch_size: 32

# Resource limits
resources:
  cpu: 1.0
  memory: "1Gi"
  gpu: 0
"#,
        name, name
    ))
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
