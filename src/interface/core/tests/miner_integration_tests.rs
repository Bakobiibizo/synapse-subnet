use super::miner_test_utils::*;
use crate::interface::core::models::miner::*;
use crate::interface::core::services::miner::*;
use std::sync::Arc;
use tokio::test;

async fn setup_test_environment() -> (Arc<MinerServiceImpl>, Arc<MockDockerClient>) {
    let db = Arc::new(create_test_db().await);
    let docker = Arc::new(MockDockerClient::new());
    let service = Arc::new(MinerServiceImpl::new(db, docker.clone()));
    (service, docker)
}

#[test]
async fn test_miner_lifecycle() {
    let (service, docker) = setup_test_environment().await;
    let module_name = "test_module";

    // Test registration
    let registration = create_test_registration(module_name);
    let status = service.register_miner(registration).await.unwrap();
    assert_miner_status(&status, false, 1000);

    // Test start mining
    let config = create_test_mining_config();
    let status = service.start_mining(module_name.to_string(), config).await.unwrap();
    assert_miner_status(&status, true, 1000);

    // Verify container was created
    let containers = docker.list_containers().await.unwrap();
    assert!(containers.contains(&module_name.to_string()));

    // Test stop mining
    let status = service.stop_mining(module_name.to_string()).await.unwrap();
    assert_miner_status(&status, false, 1000);

    // Verify container was stopped
    let containers = docker.list_containers().await.unwrap();
    assert!(!containers.contains(&module_name.to_string()));
}

#[test]
async fn test_stake_management() {
    let (service, _) = setup_test_environment().await;
    let module_name = "test_module";

    // Register miner
    let registration = create_test_registration(module_name);
    let _ = service.register_miner(registration).await.unwrap();

    // Test stake update
    let update = StakeUpdate {
        new_stake: 2000,
        auto_restake: Some(true),
    };
    let status = service.update_stake(module_name.to_string(), update).await.unwrap();
    assert_miner_status(&status, false, 2000);

    // Test invalid stake amount
    let update = StakeUpdate {
        new_stake: 0,
        auto_restake: None,
    };
    let result = service.update_stake(module_name.to_string(), update).await;
    assert!(matches!(result, Err(MinerError::InvalidStake(_))));
}

#[test]
async fn test_metrics_collection() {
    let (service, _) = setup_test_environment().await;
    let module_name = "test_module";

    // Register and start miner
    let registration = create_test_registration(module_name);
    let _ = service.register_miner(registration).await.unwrap();
    let config = create_test_mining_config();
    let _ = service.start_mining(module_name.to_string(), config).await.unwrap();

    // Get metrics for specific module
    let metrics = service.get_metrics(Some(module_name.to_string())).await.unwrap();
    assert!(!metrics.is_empty());
    if let Some(metric) = metrics.first() {
        assert_miner_metrics(metric);
    }

    // Get all metrics
    let all_metrics = service.get_metrics(None).await.unwrap();
    assert!(!all_metrics.is_empty());
}

#[test]
async fn test_error_handling() {
    let (service, _) = setup_test_environment().await;
    
    // Test non-existent module
    let result = service.start_mining("nonexistent".to_string(), create_test_mining_config()).await;
    assert!(matches!(result, Err(MinerError::ModuleNotFound(_))));

    // Test duplicate registration
    let module_name = "test_module";
    let registration = create_test_registration(module_name);
    let _ = service.register_miner(registration.clone()).await.unwrap();
    let result = service.register_miner(registration).await;
    assert!(matches!(result, Err(MinerError::DatabaseError(_))));

    // Test invalid resource limits
    let mut config = create_test_mining_config();
    config.resource_limits = Some(ResourceLimits {
        cpu_cores: 0.0,
        memory_mb: 0,
        storage_gb: 0,
    });
    let result = service.start_mining(module_name.to_string(), config).await;
    assert!(matches!(result, Err(MinerError::ResourceExceeded(_))));
}

#[test]
async fn test_concurrent_operations() {
    let (service, _) = setup_test_environment().await;
    let module_name = "test_module";

    // Register miner
    let registration = create_test_registration(module_name);
    let _ = service.register_miner(registration).await.unwrap();

    // Spawn multiple concurrent stake updates
    let mut handles = vec![];
    for i in 0..5 {
        let service = service.clone();
        let module_name = module_name.to_string();
        handles.push(tokio::spawn(async move {
            let update = StakeUpdate {
                new_stake: 1000 + i * 100,
                auto_restake: None,
            };
            service.update_stake(module_name, update).await
        }));
    }

    // Verify all operations completed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify final state
    let status = service.get_status(Some(module_name.to_string())).await.unwrap();
    assert_eq!(status.len(), 1);
}
