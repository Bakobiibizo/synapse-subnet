use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use git2::Repository;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");
const MODULES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/subnet-modules");

#[derive(Parser, Debug)]
#[clap(about = "Ingest a subnet module from a git repository")]
pub struct IngestCommand {
    /// Git repository URL to ingest. The repository should contain a Dockerfile and optionally an .env.example file.
    #[clap(long)]
    pub repo: String,

    /// Custom name for the module (defaults to repository name)
    #[clap(long)]
    pub name: Option<String>,

    /// Branch or tag to checkout (defaults to main)
    #[clap(long, default_value = "main")]
    pub branch: String,
}

impl IngestCommand {
    pub fn run(&self) -> Result<String> {
        // Create modules directory if it doesn't exist
        fs::create_dir_all(MODULES_DIR)?;
        
        // Create temporary directory for cloning
        let temp_dir = tempfile::tempdir()?;
        
        // Clone the repository
        println!("Cloning repository {}...", self.repo);
        let repo = Repository::clone(&self.repo, temp_dir.path())?;
        
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

        // Create module config directory
        let module_config_dir = PathBuf::from(CONFIG_DIR).join(&name);
        fs::create_dir_all(&module_config_dir)?;

        // Create module directory in subnet-modules
        let module_dir = PathBuf::from(MODULES_DIR).join(&name);
        if module_dir.exists() {
            fs::remove_dir_all(&module_dir)?;
        }
        
        // Copy directory contents instead of moving
        copy_dir_all(temp_dir.path(), &module_dir)?;

        // Copy .env.example if it exists
        let env_example = module_dir.join(".env.example");
        if env_example.exists() {
            fs::copy(&env_example, module_config_dir.join(".env.example"))?;
        }

        // Create config.yaml
        let config = format!(
            r#"registrar:
  endpoint: "http://localhost:8080"
  modules:
    - name: "{}"
      type: "docker"
      image: "{}"
      tag: "latest"
      port: 8081
      env: ${{ENV_VARS}}
      volumes:
        - "${{PWD}}/subnet-modules/{}:/app"

validator:
  port: 8082
  key: "${{VALIDATOR_KEY}}"
  chain_endpoint: "http://localhost:8083""#,
            name, name, name
        );
        fs::write(module_config_dir.join("config.yaml"), config)?;

        // Create install script
        let install_script = format!(
            r##"#!/bin/bash
set -e

# Configuration
MODULE_NAME="{}"
DOCKER_IMAGE="{}:latest"
VALIDATOR_PORT="${{VALIDATOR_PORT:-8081}}"
REGISTRAR_URL="${{REGISTRAR_URL:-http://localhost:8080}}"

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${{BASH_SOURCE[0]}}" )" &> /dev/null && pwd )"

# Create and setup subnet directory
echo "Creating subnet directory..."
SUBNET_DIR="$SCRIPT_DIR/subnet/$MODULE_NAME"
mkdir -p "$SUBNET_DIR"

# Download module configuration
echo "Downloading module configuration..."
curl -s "$REGISTRAR_URL/api/v1/modules/$MODULE_NAME/config" -o "$SUBNET_DIR/config.yaml"

# Download environment template
echo "Downloading environment template..."
curl -s "$REGISTRAR_URL/api/v1/modules/$MODULE_NAME/env-template" -o "$SUBNET_DIR/.env.example"

# Create environment file if it doesn't exist
if [ ! -f "$SUBNET_DIR/.env" ]; then
    echo "Setting up environment variables..."
    while IFS='=' read -r key value; do
        # Skip comments and empty lines
        [[ $key =~ ^# ]] && continue
        [[ -z $key ]] && continue
        
        # Remove quotes from value
        value=$(echo "$value" | tr -d '"')
        
        # Prompt for value
        read -p "$key [$value]: " input
        echo "$key=${{input:-$value}}" >> "$SUBNET_DIR/.env"
    done < "$SUBNET_DIR/.env.example"
fi

# Pull Docker image
echo "Pulling Docker image..."
docker pull "$REGISTRAR_URL/v2/$MODULE_NAME"

# Start validator
echo "Starting validator..."
docker run -d \
    --name "$MODULE_NAME-validator" \
    --restart unless-stopped \
    --env-file "$SUBNET_DIR/.env" \
    -v "$SUBNET_DIR/config.yaml:/app/config.yaml" \
    -p "$VALIDATOR_PORT:8081" \
    "$DOCKER_IMAGE"

echo "Validator is running!"
echo "Subnet Directory: $SUBNET_DIR"
echo "Port: $VALIDATOR_PORT"
echo "Config: $SUBNET_DIR/config.yaml"
echo "Logs: docker logs -f $MODULE_NAME-validator"

# Create helper scripts
cat > "$SUBNET_DIR/start.sh" << 'EOF'
#!/bin/bash
docker start $MODULE_NAME-validator
EOF
chmod +x "$SUBNET_DIR/start.sh"

cat > "$SUBNET_DIR/stop.sh" << 'EOF'
#!/bin/bash
docker stop $MODULE_NAME-validator
EOF
chmod +x "$SUBNET_DIR/stop.sh"

cat > "$SUBNET_DIR/logs.sh" << 'EOF'
#!/bin/bash
docker logs -f $MODULE_NAME-validator
EOF
chmod +x "$SUBNET_DIR/logs.sh"

echo "Helper scripts created:"
echo "  $SUBNET_DIR/start.sh  - Start the validator"
echo "  $SUBNET_DIR/stop.sh   - Stop the validator"
echo "  $SUBNET_DIR/logs.sh   - View validator logs"
"##,
            name, name
        );
        let install_script_path = module_config_dir.join("install.sh");
        fs::write(&install_script_path, install_script)?;
        
        // Set execute permissions (0o755)
        let mut perms = fs::metadata(&install_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_script_path, perms)?;

        println!("Module '{}' ingested successfully!", name);
        Ok(name)
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
