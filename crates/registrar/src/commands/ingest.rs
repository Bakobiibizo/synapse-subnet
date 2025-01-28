use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use tokio::process::Command;
use std::os::unix::fs::PermissionsExt;

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
        
        // Create a known directory for cloning
        let temp_dir = std::path::PathBuf::from("/home/administrator/tmp/module_clone");
        std::fs::create_dir_all(&temp_dir)?;
        
        // Explicitly set permissions
        std::fs::set_permissions(&temp_dir, std::fs::Permissions::from_mode(0o777))?;
        
        // Debug logging
        println!("Temp directory: {}", temp_dir.display());
        println!("Temp directory permissions: {:?}", std::fs::metadata(&temp_dir)?.permissions());
        println!("Current user: {}", std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()));
        
        // Clone the repository using git command
        println!("Cloning repository {}...", self.repo);
        let status = Command::new("git")
            .current_dir(&temp_dir)  // Explicitly set current directory
            .args([
                "clone",
                "--depth", "1",
                "--branch", &self.branch,
                &self.repo,
                "."  // Clone directly into the temp directory
            ])
            .status()
            .await?;
            
        if !status.success() {
            anyhow::bail!("Failed to clone repository");
        }
        
        // Get module name from repo URL or custom name
        let module_name = match &self.name {
            Some(name) => name.clone(),
            None => {
                let repo_parts: Vec<&str> = self.repo.split('/').collect();
                repo_parts[repo_parts.len() - 1]
                    .replace(".git", "")
                    .to_lowercase()
            }
        };

        // Copy to modules directory
        let module_dir = PathBuf::from(MODULES_DIR).join(&module_name);
        std::fs::create_dir_all(&module_dir)?;
        copy_dir_all(&temp_dir, &module_dir)?;

        Ok(module_name)
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
