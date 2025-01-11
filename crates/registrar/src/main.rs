use clap::{Parser, Subcommand};
mod commands;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the registrar server
    Start(commands::start::StartCommand),
    
    /// Ingest a module from a git repository
    Ingest(commands::ingest::IngestCommand),

    /// Manage subnet environments
    Env(commands::env::EnvCommand),

    /// Install and run a subnet module
    Install(commands::install::InstallCommand),

    /// Register a new subnet module (ingest, setup environment, and install)
    Register(commands::register::RegisterCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start(cmd) => cmd.run().await,
        Commands::Ingest(cmd) => {
            cmd.run()?;
            Ok(())
        },
        Commands::Env(cmd) => cmd.run(),
        Commands::Install(cmd) => cmd.run().await,
        Commands::Register(cmd) => cmd.run().await,
    }
}
