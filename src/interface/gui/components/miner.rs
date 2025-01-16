//! Miner management components

use super::*;
use crate::interface::core::models::{MinerStatus, MinerMetrics, MiningConfig, StakeUpdate};

/// Miner dashboard component
#[derive(Template)]
#[template(path = "components/miner/dashboard.html")]
pub struct MinerDashboard {
    /// Overall miner status
    pub status: MinerStatus,
    /// Performance metrics
    pub metrics: MinerMetrics,
    /// Active modules
    pub active_modules: Vec<String>,
    /// Total stake
    pub total_stake: u64,
}

/// Module mining status component
#[derive(Template)]
#[template(path = "components/miner/module_status.html")]
pub struct ModuleMiningStatus {
    /// Module name
    pub name: String,
    /// Mining status
    pub status: MinerStatus,
    /// Module metrics
    pub metrics: MinerMetrics,
    /// Staked amount
    pub stake: u64,
}

/// Mining configuration form
#[derive(Template)]
#[template(path = "components/miner/config_form.html")]
pub struct MiningConfigForm {
    /// Current configuration
    pub config: MiningConfig,
    /// Available balance
    pub available_balance: u64,
    /// Form errors
    pub errors: Vec<String>,
}

/// Stake management form
#[derive(Template)]
#[template(path = "components/miner/stake_form.html")]
pub struct StakeForm {
    /// Module name
    pub module: String,
    /// Current stake
    pub current_stake: u64,
    /// Available balance
    pub available_balance: u64,
    /// Form errors
    pub errors: Vec<String>,
}

impl Component for MinerDashboard {}
impl Component for ModuleMiningStatus {}
impl Component for MiningConfigForm {}
impl Component for StakeForm {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    /// Test dashboard rendering
    #[tokio::test]
    async fn test_dashboard_render() {
        let component = MinerDashboard {
            status: MinerStatus {
                is_active: true,
                uptime: 3600,
                last_update: chrono::Utc::now(),
            },
            metrics: MinerMetrics {
                total_blocks: 100,
                success_rate: 0.95,
                average_block_time: 150,
                rewards_earned: 1000,
            },
            active_modules: vec!["test_module".to_string()],
            total_stake: 5000,
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that response contains key metrics
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("95%")); // Success rate
        assert!(html.contains("5000")); // Total stake
    }

    /// Test stake form validation
    #[tokio::test]
    async fn test_stake_form_validation() {
        let component = StakeForm {
            module: "test_module".to_string(),
            current_stake: 1000,
            available_balance: 5000,
            errors: vec!["Insufficient balance".to_string()],
        };

        let response = component.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test that response contains error message
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Insufficient balance"));
    }
}
