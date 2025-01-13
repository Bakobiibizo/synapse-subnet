use std::path::PathBuf;
use sqlx::{Row, SqlitePool, FromRow};
use anyhow::{Result, Context};
use time::OffsetDateTime;
use sha2::{Sha256, Digest};
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Serialize, Deserialize};
use registrar_core::{Module, ModuleType, ModuleStatus};
use thiserror::Error;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Package error: {0}")]
    Package(String),
}

#[derive(Serialize, Deserialize)]
pub struct PackageFile {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub repo_url: String,
    pub branch: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub created_at: String,
    pub package_hash: String,
    pub files: Vec<PackageFile>,
}

#[derive(Serialize)]
pub struct ModulePackage {
    pub package: String,
    pub hash: String,
    pub metadata: RegistryModule,
}

pub struct Registry {
    db: SqlitePool,
    config_dir: PathBuf,
}

impl Registry {
    pub async fn new(db_url: &str, config_dir: impl Into<PathBuf>) -> Result<Self> {
        // Create database directory if it doesn't exist
        let db_path = PathBuf::from(db_url.trim_start_matches("sqlite:"));
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Connect to database with better connection options
        let pool_options = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5);

        // Add SQLite connection options for better reliability
        let db_url_with_opts = format!(
            "sqlite:{}?mode=rwc&cache=shared",
            db_path.display()
        );

        let db = pool_options
            .connect(&db_url_with_opts)
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        sqlx::migrate!()
            .run(&db)
            .await
            .context("Failed to run database migrations")?;

        Ok(Self {
            db,
            config_dir: config_dir.into(),
        })
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub async fn create_module(&self, module: &RegistryModule) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO subnet_modules (
                name, version, repo_url, branch, description, author, license,
                created_at, updated_at, downloads, module_type, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&module.name)
        .bind(&module.version)
        .bind(&module.repo_url)
        .bind(&module.branch)
        .bind(&module.description)
        .bind(&module.author)
        .bind(&module.license)
        .bind(module.created_at)
        .bind(module.updated_at)
        .bind(module.downloads)
        .bind(&module.module_type)
        .bind(&module.status)
        .fetch_one(&self.db)
        .await
        .context("Failed to insert module")?;

        let id: i64 = result.get(0);
        Ok(id)
    }

    pub async fn get_module(&self, name: &str) -> Result<Option<RegistryModule>> {
        let module = sqlx::query_as::<_, RegistryModule>(
            r#"
            SELECT 
                id, 
                name, 
                version, 
                repo_url, 
                branch, 
                description,
                author,
                license,
                created_at,
                updated_at,
                downloads,
                module_type,
                status
            FROM subnet_modules
            WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_optional(&self.db)
        .await
        .context("Failed to get module")?;

        Ok(module)
    }

    pub async fn list_modules(&self) -> Result<Vec<RegistryModule>> {
        let modules = sqlx::query_as::<_, RegistryModule>(
            r#"
            SELECT 
                id, 
                name, 
                version, 
                repo_url, 
                branch, 
                description,
                author,
                license,
                created_at,
                updated_at,
                downloads,
                module_type,
                status
            FROM subnet_modules
            "#,
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to list modules")?;

        Ok(modules)
    }

    pub async fn increment_downloads(&self, name: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE subnet_modules SET downloads = downloads + 1 WHERE name = ?
            "#,
        )
        .bind(name)
        .execute(&self.db)
        .await
        .context("Failed to increment downloads")?;

        Ok(())
    }

    pub async fn get_package(&self, name: &str) -> Result<ModulePackage> {
        let module = self.get_module(name).await?
            .ok_or_else(|| RegistryError::ModuleNotFound(name.to_string()))?;

        let package_path = self.config_dir.join(name).join("package.tar.gz");
        let package_bytes = std::fs::read(&package_path)?;

        let package = general_purpose::STANDARD.encode(&package_bytes);
        let mut hasher = Sha256::new();
        hasher.update(&package_bytes);
        let hash = general_purpose::STANDARD.encode(hasher.finalize());

        Ok(ModulePackage {
            package,
            hash,
            metadata: module,
        })
    }
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct RegistryModule {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub repo_url: String,
    pub branch: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub downloads: i64,
    pub module_type: String,
    pub status: String,
}

impl RegistryModule {
    pub fn get_module_type(&self) -> ModuleType {
        ModuleType::from_str(&self.module_type).unwrap_or(ModuleType::Observer)
    }

    pub fn get_status(&self) -> ModuleStatus {
        ModuleStatus::from_str(&self.status).unwrap_or(ModuleStatus::Stopped)
    }
}

impl From<RegistryModule> for Module {
    fn from(rm: RegistryModule) -> Self {
        let module_type = ModuleType::from_str(&rm.module_type).unwrap_or(ModuleType::Observer);
        let status = ModuleStatus::from_str(&rm.status).unwrap_or(ModuleStatus::Stopped);
        Self {
            name: rm.name,
            module_type,
            status,
        }
    }
}

impl From<Module> for RegistryModule {
    fn from(m: Module) -> Self {
        Self {
            id: 0, // This will be set by the database
            name: m.name,
            version: String::new(),
            repo_url: String::new(),
            branch: String::new(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            downloads: 0,
            module_type: m.module_type.to_string(),
            status: m.status.to_string(),
        }
    }
}

impl Clone for RegistryModule {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            version: self.version.clone(),
            repo_url: self.repo_url.clone(),
            branch: self.branch.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            downloads: self.downloads,
            module_type: self.module_type.clone(),
            status: self.status.clone(),
        }
    }
}
