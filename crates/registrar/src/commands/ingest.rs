use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use registrar_core::{ModuleType, ModuleStatus};
use time::OffsetDateTime;

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
        
        // Clone the repository
        println!("Cloning repository {}...", self.repo);
        let repo = git2::Repository::clone(&self.repo, temp_dir.path())?;
        
        // Checkout specified branch
        let obj = repo.revparse_single(&format!("origin/{}", self.branch))?;
        repo.checkout_tree(&obj, None)?;
        
        // Get module name from repo URL or custom name
        let name = self.name.clone().unwrap_or_else(|| {
            self.repo
                .split('/')
                .last()
                .unwrap()
                .trim_end_matches(".git")
                .to_string()
        });

        // Validate module structure
        let temp_path = temp_dir.path();
        if !temp_path.join("Dockerfile").exists() {
            anyhow::bail!("Module must contain a Dockerfile");
        }

        // Create module config directory
        let module_config_dir = PathBuf::from(CONFIG_DIR).join(&name);
        std::fs::create_dir_all(&module_config_dir)?;

        // Create module directory in subnet-modules
        let module_dir = PathBuf::from(CONFIG_DIR).join(&name);
        if module_dir.exists() {
            std::fs::remove_dir_all(&module_dir)?;
        }
        
        // Copy directory contents
        copy_dir_all(temp_dir.path(), &module_dir)?;

        // Create or copy .env.example
        let env_example = if temp_path.join(".env.example").exists() {
            std::fs::read_to_string(temp_path.join(".env.example"))?
        } else {
            create_default_env_template(&name)
        };
        std::fs::write(module_config_dir.join(".env.example"), env_example)?;

        // Create config.yaml
        let config = create_config_yaml(&name)?;
        std::fs::write(module_config_dir.join("config.yaml"), config)?;

        let db_url = format!("sqlite:{}/data/registrar.db", env!("CARGO_MANIFEST_DIR"));
        let registry = Registry::new(&db_url, &module_dir).await?;

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
        println!("\nTo install this module on a validator, run:");
        println!("cargo run -p validator -- install --name {} --registrar-url http://localhost:8080", name);
        
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
