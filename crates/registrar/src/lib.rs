//! Registrar implementation for the Synapse Subnet project.
//! 
//! This crate provides the module registry and build system for managing
//! inference modules, supporting both Docker-based and native implementations.

use std::path::PathBuf;
use anyhow::Result;
use sqlx::SqlitePool;
use time::OffsetDateTime;

pub mod interface;
pub mod module;
pub mod registry;
pub mod docker;
pub mod verification;
pub mod commands;
pub mod api;

pub use interface::{
    Health, HealthStatus, InferenceModule, InferenceParameters,
    Input, MetricsData, ModuleCapabilities, Output, ResourceRequirements,
    TokenUsage,
};
pub use module::{Module, ModuleType, ModuleStatus, ModuleState, ModuleRuntime};
pub use registry::{Registry, RegistryError};
pub use docker::DockerModuleRuntime;
pub use verification::{ModuleVerifier, VerificationConfig, VerificationError};

pub struct Registrar {
    db: SqlitePool,
    config_dir: PathBuf,
}

impl Registrar {
    pub async fn new(db_url: &str, config_dir: impl Into<PathBuf>) -> Result<Self> {
        let db = SqlitePool::connect(db_url).await?;
        Ok(Self {
            db,
            config_dir: config_dir.into(),
        })
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct RegistryModule {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub repo_url: String,
    pub branch: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub downloads: i64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
