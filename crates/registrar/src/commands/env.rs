use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use clap::Parser;
use anyhow::{Result, Context};
use dialoguer::{Input, Confirm};
use std::fmt::Write as _;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

#[derive(Parser, Debug)]
#[clap(about = "Manage subnet environments")]
pub struct EnvCommand {
    #[clap(subcommand)]
    command: EnvSubCommand,
}

#[derive(Parser, Debug)]
enum EnvSubCommand {
    /// Create a new subnet environment from scratch or from a template
    Create(CreateEnvCommand),
    /// Activate a subnet environment for use
    Activate(ActivateEnvCommand),
    /// List all available subnet environments
    List(ListEnvCommand),
}

#[derive(Parser, Debug)]
#[clap(about = "Create a new subnet environment")]
pub struct CreateEnvCommand {
    /// Name of the subnet environment to create
    name: String,
    /// Path to .env.example file to use as template. If not provided, a default template will be used.
    #[clap(long)]
    from_example: Option<PathBuf>,
    /// Accept default values without prompting. Useful for automated installations.
    #[clap(long)]
    accept_defaults: bool,
}

#[derive(Parser, Debug)]
#[clap(about = "Activate a subnet environment")]
pub struct ActivateEnvCommand {
    /// Name of the subnet environment to activate
    name: String,
}

#[derive(Parser, Debug)]
#[clap(about = "List all available subnet environments")]
pub struct ListEnvCommand {
    /// Show the values of environment variables in each environment
    #[clap(long)]
    show_values: bool,
}

impl EnvCommand {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            EnvSubCommand::Create(cmd) => cmd.run(),
            EnvSubCommand::Activate(cmd) => cmd.run(),
            EnvSubCommand::List(cmd) => cmd.run(),
        }
    }
}

impl CreateEnvCommand {
    pub fn run(&self) -> Result<()> {
        let env_dir = get_env_dir()?;
        let env_file = env_dir.join(format!("{}.env", self.name));

        if env_file.exists() && !self.accept_defaults {
            let overwrite = Confirm::new()
                .with_prompt(format!("Environment for subnet '{}' already exists. Overwrite?", self.name))
                .default(false)
                .interact()?;
            
            if !overwrite {
                println!("Aborted.");
                return Ok(());
            }
        }

        if let Some(example_path) = &self.from_example {
            self.create_from_example(example_path, &env_file)?;
        } else {
            self.create_default(&env_file)?;
        }

        println!("Created environment file at {:?}", env_file);
        Ok(())
    }

    fn create_from_example(&self, example_path: &Path, env_file: &Path) -> Result<()> {
        let example = File::open(example_path)
            .context("Failed to open example file")?;
        let reader = BufReader::new(example);
        let mut env_content = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                writeln!(env_content, "{}", line)?;
                continue;
            }

            if let Some((key, default_value)) = parse_env_line(&line) {
                let value = if self.accept_defaults {
                    default_value
                } else {
                    Input::new()
                        .with_prompt(format!("{} [default: {}]", key, default_value))
                        .default(default_value)
                        .interact_text()?
                };
                
                writeln!(env_content, "{}={}", key, value)?;
            } else {
                writeln!(env_content, "{}", line)?;
            }
        }

        fs::write(env_file, env_content)?;
        Ok(())
    }

    fn create_default(&self, env_file: &Path) -> Result<()> {
        let mut default_content = String::new();
        writeln!(default_content, "# Environment variables for {} subnet", self.name)?;
        writeln!(default_content, "# Edit these values with your API keys")?;
        writeln!(default_content, "")?;
        writeln!(default_content, "# Required API Keys")?;
        writeln!(default_content, "OPENAI_API_KEY=")?;
        writeln!(default_content, "ANTHROPIC_API_KEY=")?;
        writeln!(default_content, "HUGGINGFACE_API_KEY=")?;
        writeln!(default_content, "")?;
        writeln!(default_content, "# Validator Configuration")?;
        writeln!(default_content, "VALIDATOR_KEY=")?;
        writeln!(default_content, "VALIDATOR_PORT=8081")?;
        writeln!(default_content, "CHAIN_ENDPOINT=http://localhost:8083")?;
        writeln!(default_content, "")?;
        writeln!(default_content, "# Custom Environment Variables")?;
        writeln!(default_content, "# Add any subnet-specific variables below")?;

        fs::write(env_file, default_content)?;
        Ok(())
    }
}

impl ActivateEnvCommand {
    pub fn run(&self) -> Result<()> {
        let env_dir = get_env_dir()?;
        let env_file = env_dir.join(format!("{}.env", self.name));
        let active_link = env_dir.join("active.env");

        if !env_file.exists() {
            anyhow::bail!("Environment for subnet '{}' not found", self.name);
        }

        if active_link.exists() {
            fs::remove_file(&active_link)?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&env_file, &active_link)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&env_file, &active_link)?;

        println!("Activated environment for subnet '{}'", self.name);
        Ok(())
    }
}

impl ListEnvCommand {
    pub fn run(&self) -> Result<()> {
        let env_dir = get_env_dir()?;
        let active_env = env_dir.join("active.env");
        let active_target = if active_env.exists() {
            fs::read_link(&active_env).ok()
        } else {
            None
        };

        println!("Available subnet environments:");
        for entry in fs::read_dir(&env_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("env") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if name != "active" {
                        let is_active = active_target.as_ref().map_or(false, |t| t == &path);
                        let prefix = if is_active { "* " } else { "  " };
                        
                        if self.show_values {
                            let vars = load_env_file(&path)?;
                            println!("{}{}:", prefix, name);
                            for (key, value) in vars {
                                println!("    {}={}", key, value);
                            }
                        } else {
                            println!("{}{}{}", prefix, name, if is_active { " (active)" } else { "" });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_env_dir() -> Result<PathBuf> {
    let env_dir = PathBuf::from(CONFIG_DIR).join(".env");
    fs::create_dir_all(&env_dir)?;
    Ok(env_dir)
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let value = parts[1].trim().trim_matches('"').to_string();
        Some((key, value))
    } else {
        None
    }
}

fn load_env_file(path: &Path) -> Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut vars = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((key, value)) = parse_env_line(&line) {
            vars.insert(key, value);
        }
    }

    Ok(vars)
}
