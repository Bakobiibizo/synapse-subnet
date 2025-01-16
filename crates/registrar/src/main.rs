use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, util::SubscriberInitExt};

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

mod api;
mod registry;
mod commands;

use registry::Registry;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Start the registrar server
    Serve(ServeCommand),
    /// Ingest a subnet module from a git repository
    Ingest(commands::ingest::IngestCommand),
    /// Show environment variables
    Env(commands::env::EnvCommand),
    /// Install a subnet module
    Install(commands::install::InstallCommand),
    /// Register a subnet module
    Register(commands::register::RegisterCommand),
}

#[derive(Parser, Debug)]
#[clap(about = "Start the registrar server")]
pub struct ServeCommand {
    /// Port to listen on
    #[clap(long, default_value = "8080")]
    pub port: u16,

    /// Host to bind to
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Config directory
    #[clap(long, default_value = CONFIG_DIR)]
    pub config_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(cmd) => serve(cmd).await,
        Commands::Ingest(cmd) => cmd.run().await.map(|_| ()),
        Commands::Env(cmd) => cmd.run(),
        Commands::Install(cmd) => cmd.run().await,
        Commands::Register(cmd) => cmd.run().await,
    }
}

async fn serve(cmd: ServeCommand) -> Result<()> {
    let addr = format!("{}:{}", cmd.host, cmd.port).parse::<SocketAddr>()?;

    // Initialize database with proper permissions
    let db_path = std::env::current_dir()?.join("data/registrar.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
        std::fs::set_permissions(
            parent,
            std::fs::Permissions::from_mode(0o777),
        )?;
    }
    
    // Remove existing database to start fresh
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
    }
    
    let db_url = format!("sqlite:{}", db_path.display());
    info!("Using database at: {}", db_url);
    let registry = std::sync::Arc::new(crate::registry::Registry::new(&db_url, &cmd.config_dir).await?);

    // Build application
    let app = axum::Router::new()
        .nest("/api", crate::api::modules::router(registry))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    info!("Starting server on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app,
    )
    .await?;

    Ok(())
}
