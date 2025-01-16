use super::*;

#[test]
fn test_miner_config_creation() {
    let config = MinerConfig {
        module_name: "test_module".to_string(),
        stake_amount: 1000,
        auto_restake: true,
        priority_level: PriorityLevel::High,
        resource_limits: ResourceLimits {
            cpu_cores: 2.0,
            memory_mb: 512,
            storage_gb: 5,
        },
    };

    assert_eq!(config.module_name, "test_module");
    assert_eq!(config.stake_amount, 1000);
    assert!(config.auto_restake);
    assert_eq!(config.priority_level, PriorityLevel::High);
    assert_eq!(config.resource_limits.cpu_cores, 2.0);
    assert_eq!(config.resource_limits.memory_mb, 512);
    assert_eq!(config.resource_limits.storage_gb, 5);
}

#[test]
fn test_resource_limits_validation() {
    let valid_limits = ResourceLimits {
        cpu_cores: 2.0,
        memory_mb: 512,
        storage_gb: 5,
    };
    assert!(valid_limits.validate().is_ok());

    let invalid_limits = ResourceLimits {
        cpu_cores: 0.0,
        memory_mb: 0,
        storage_gb: 0,
    };
    assert!(invalid_limits.validate().is_err());
}

#[test]
fn test_priority_level_ordering() {
    assert!(PriorityLevel::High > PriorityLevel::Medium);
    assert!(PriorityLevel::Medium > PriorityLevel::Low);
    assert!(PriorityLevel::Low > PriorityLevel::Background);
}
