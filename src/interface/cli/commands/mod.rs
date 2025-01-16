//! CLI command implementations

pub mod registrar;
pub mod validator;
pub mod miner;
pub mod keys;

use crate::interface::core::{auth, db, environment};
use clap::Subcommand;

/// Trait for command execution
#[async_trait::async_trait]
pub trait Command {
    /// Execute the command
    async fn execute(self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Commands for managing the registrar
pub mod registrar {
    use super::*;

    #[derive(Subcommand)]
    pub enum RegistrarCommands {
        /// Register a new module
        Register {
            /// Path to module directory
            path: String,
            /// Optional version tag
            #[arg(short, long)]
            version: Option<String>,
        },

        /// Unregister a module
        Unregister {
            /// Module name
            name: String,
        },

        /// List registered modules
        List,

        /// Update module status
        UpdateStatus {
            /// Module name
            name: String,
            /// New status
            status: String,
        },
    }
}

/// Commands for managing validators
pub mod validator {
    use super::*;

    #[derive(Subcommand)]
    pub enum ValidatorCommands {
        /// Start validation
        Start {
            /// Port to listen on
            #[arg(short, long, default_value = "8080")]
            port: u16,
            /// Optional registrar URL
            #[arg(short, long)]
            registrar_url: Option<String>,
        },

        /// Stop validation
        Stop,

        /// Show validator status
        Status,

        /// Configure validator settings
        Configure {
            /// Configuration file path
            config: String,
        },
    }
}

/// Commands for managing miners
pub mod miner {
    use super::*;

    #[derive(Subcommand)]
    pub enum MinerCommands {
        /// Register as a miner
        Register {
            /// Miner name
            name: String,
            /// SS58 key
            key: String,
        },

        /// Start mining
        Start {
            /// Module to mine
            module: String,
            /// Optional stake amount
            #[arg(short, long)]
            stake: Option<u64>,
        },

        /// Stop mining
        Stop {
            /// Module name
            module: String,
        },

        /// Show miner status
        Status {
            /// Optional module filter
            #[arg(short, long)]
            module: Option<String>,
        },
    }
}

/// Commands for managing SS58 keys
pub mod keys {
    use super::*;

    #[derive(Subcommand)]
    pub enum KeyCommands {
        /// Generate a new key pair
        Generate {
            /// Optional key name
            #[arg(short, long)]
            name: Option<String>,
        },

        /// Import an existing key
        Import {
            /// Path to key file
            path: String,
            /// Optional key name
            #[arg(short, long)]
            name: Option<String>,
        },

        /// Export a key
        Export {
            /// Key name or address
            key: String,
            /// Export path
            path: String,
        },

        /// List all keys
        List,

        /// Sign a message
        Sign {
            /// Message to sign
            message: String,
            /// Key to use
            key: String,
        },
    }
}
