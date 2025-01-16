use crate::interface::core::models::miner::*;
use crate::interface::core::services::miner::*;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create a test miner registration
pub fn create_test_registration(module_name: &str) -> MinerRegistration {
    MinerRegistration {
        module_name: module_name.to_string(),
        initial_stake: 1000,
        config: MinerConfig {
            module_name: module_name.to_string(),
            stake_amount: 1000,
            auto_restake: true,
            priority_level: PriorityLevel::Medium,
            resource_limits: ResourceLimits::default(),
        },
    }
}

/// Create a test mining config
pub fn create_test_mining_config() -> MiningConfig {
    MiningConfig {
        auto_restake: Some(true),
        priority_level: Some(PriorityLevel::High),
        resource_limits: Some(ResourceLimits {
            cpu_cores: 1.0,
            memory_mb: 256,
            storage_gb: 1,
        }),
    }
}

/// Create test metrics
pub fn create_test_metrics() -> MinerMetrics {
    MinerMetrics {
        total_blocks: 100,
        success_rate: 0.95,
        average_block_time: 5000,
        rewards_earned: 5000,
        last_block_timestamp: Utc::now(),
    }
}

/// Mock Docker client for testing
#[derive(Default)]
pub struct MockDockerClient {
    pub containers: Arc<RwLock<Vec<String>>>,
}

impl MockDockerClient {
    pub fn new() -> Self {
        Self {
            containers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_container(&self, name: &str) -> Result<(), String> {
        let mut containers = self.containers.write().await;
        containers.push(name.to_string());
        Ok(())
    }

    pub async fn stop_container(&self, name: &str) -> Result<(), String> {
        let mut containers = self.containers.write().await;
        if let Some(pos) = containers.iter().position(|x| x == name) {
            containers.remove(pos);
            Ok(())
        } else {
            Err("Container not found".to_string())
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<String>, String> {
        Ok(self.containers.read().await.clone())
    }
}

/// Create an in-memory SQLite database for testing
pub async fn create_test_db() -> sqlx::Pool<sqlx::Sqlite> {
    let db = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS miners (
            module_name TEXT PRIMARY KEY,
            stake_amount INTEGER NOT NULL,
            auto_restake BOOLEAN NOT NULL,
            priority_level TEXT NOT NULL,
            is_active BOOLEAN NOT NULL,
            uptime INTEGER NOT NULL,
            last_update TEXT NOT NULL
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("Failed to create miners table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS miner_metrics (
            module_name TEXT NOT NULL,
            total_blocks INTEGER NOT NULL,
            success_rate REAL NOT NULL,
            average_block_time INTEGER NOT NULL,
            rewards_earned INTEGER NOT NULL,
            last_block_timestamp TEXT NOT NULL,
            FOREIGN KEY(module_name) REFERENCES miners(module_name)
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("Failed to create miner_metrics table");

    db
}

/// Test helper to verify miner status
pub fn assert_miner_status(status: &ModuleStatus, expected_active: bool, expected_stake: u64) {
    assert_eq!(status.is_active, expected_active);
    assert_eq!(status.current_stake, expected_stake);
    assert!(status.last_update <= Utc::now());
}

/// Test helper to verify miner metrics
pub fn assert_miner_metrics(metrics: &MinerMetrics) {
    assert!(metrics.total_blocks > 0);
    assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
    assert!(metrics.average_block_time > 0);
    assert!(metrics.rewards_earned >= 0);
    assert!(metrics.last_block_timestamp <= Utc::now());
}
