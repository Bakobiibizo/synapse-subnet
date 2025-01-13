use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::str::FromStr;
use sqlx::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleType {
    Validator,
    Observer,
}

impl FromStr for ModuleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "validator" => Ok(ModuleType::Validator),
            "observer" => Ok(ModuleType::Observer),
            _ => Err(format!("Invalid module type: {}", s)),
        }
    }
}

impl ToString for ModuleType {
    fn to_string(&self) -> String {
        match self {
            ModuleType::Validator => "validator".to_string(),
            ModuleType::Observer => "observer".to_string(),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ModuleType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Decode<'_, sqlx::Sqlite> for ModuleType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        ModuleType::from_str(&s).map_err(|e| sqlx::error::BoxDynError::from(format!("{}", e)))
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for ModuleType {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(self.to_string().into()));
        sqlx::encode::IsNull::No
    }
}

impl From<String> for ModuleType {
    fn from(s: String) -> Self {
        ModuleType::from_str(&s).unwrap_or(ModuleType::Observer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleStatus {
    Running,
    Stopped,
    Failed,
}

impl FromStr for ModuleStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "running" => Ok(ModuleStatus::Running),
            "stopped" => Ok(ModuleStatus::Stopped),
            "failed" => Ok(ModuleStatus::Failed),
            _ => Err(format!("Invalid module status: {}", s)),
        }
    }
}

impl ToString for ModuleStatus {
    fn to_string(&self) -> String {
        match self {
            ModuleStatus::Running => "running".to_string(),
            ModuleStatus::Stopped => "stopped".to_string(),
            ModuleStatus::Failed => "failed".to_string(),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for ModuleStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Decode<'_, sqlx::Sqlite> for ModuleStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        ModuleStatus::from_str(&s).map_err(|e| sqlx::error::BoxDynError::from(format!("{}", e)))
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for ModuleStatus {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(self.to_string().into()));
        sqlx::encode::IsNull::No
    }
}

impl From<String> for ModuleStatus {
    fn from(s: String) -> Self {
        ModuleStatus::from_str(&s).unwrap_or(ModuleStatus::Stopped)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub module_type: ModuleType,
    pub status: ModuleStatus,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Module already exists: {0}")]
    ModuleExists(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait::async_trait]
pub trait Registry: Send + Sync {
    async fn get_module(&self, name: &str) -> Result<Option<Module>, RegistryError>;
    async fn list_modules(&self) -> Result<Vec<Module>, RegistryError>;
    async fn create_module(&self, module: &Module) -> Result<i64, RegistryError>;
    async fn update_module_status(&self, name: &str, status: ModuleStatus) -> Result<(), RegistryError>;
    async fn unregister_module(&self, name: &str) -> Result<(), RegistryError>;
    async fn start_module(&self, name: &str) -> Result<(), RegistryError>;
}
