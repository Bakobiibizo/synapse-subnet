//! Command Line Interface for Synapse Subnet
//! 
//! Provides command-line tools for interacting with registrar, validators, and miners

use clap::{Parser, Subcommand};
use crate::interface::core::prelude::*;

mod commands;
mod config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Registrar management commands
    Registrar(commands::registrar::RegistrarCommands),
    
    /// Validator management commands
    Validator(commands::validator::ValidatorCommands),
    
    /// Miner management commands
    Miner(commands::miner::MinerCommands),
    
    /// Key management commands
    Keys(commands::keys::KeyCommands),
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Registrar(cmd) => cmd.execute().await,
            Commands::Validator(cmd) => cmd.execute().await,
            Commands::Miner(cmd) => cmd.execute().await,
            Commands::Keys(cmd) => cmd.execute().await,
        }
    }
}
