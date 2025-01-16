//! Error types for environment management

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Docker error: {0}")]
    DockerError(#[from] docker_manager::DockerError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
