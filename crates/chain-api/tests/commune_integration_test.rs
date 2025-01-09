use chain_api::commune::{CommuneInterface, CommuneModule, CommuneRPC};
use std::env;

struct TestFixture {
    rpc: CommuneRPC,
}

impl TestFixture {
    fn new(use_testnet: bool) -> Self {
        // Set up Python environment
        let python_path = env::var("VIRTUAL_ENV")
            .map(|venv| format!("{}/bin/python", venv))
            .unwrap_or_else(|_| "python3".to_string());
        
        let rpc = if use_testnet {
            CommuneRPC::testnet(python_path)
        } else {
            CommuneRPC::mainnet(python_path)
        };
        
        Self { rpc }
    }
}

// Helper function to create a test module
fn create_test_module(name: &str) -> CommuneModule {
    CommuneModule {
        name: name.to_string(),
        address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(), // Test address
        stake: 1000,
        metadata: Some("Test module metadata".to_string()),
    }
}

#[test]
fn test_read_operations() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test listing modules
    let result = fixture.rpc.list_modules(netuid);
    assert!(result.is_ok(), "Failed to list modules: {:?}", result.err());
    let modules = result.unwrap();
    
    // If there are any modules, test getting one
    if let Some(first_module) = modules.first() {
        let result = fixture.rpc.get_module(&first_module.name, netuid);
        assert!(result.is_ok(), "Failed to get existing module: {:?}", result.err());
        let retrieved = result.unwrap();
        assert!(retrieved.is_some(), "Module not found");
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, first_module.name);
    }

    // Test getting non-existent module
    let result = fixture.rpc.get_module("non_existent_module", netuid);
    assert!(result.is_ok(), "Get module should not error for non-existent module");
    assert!(result.unwrap().is_none(), "Non-existent module should return None");

    // Test getting minimum stake
    let min_stake = fixture.rpc.get_min_stake(netuid);
    assert!(min_stake.is_ok(), "Failed to get minimum stake: {:?}", min_stake.err());
    assert!(min_stake.unwrap() > 0, "Minimum stake should be greater than 0");

    // Test getting maximum allowed modules
    let max_modules = fixture.rpc.get_max_allowed_modules();
    assert!(max_modules.is_ok(), "Failed to get max allowed modules: {:?}", max_modules.err());
    assert!(max_modules.unwrap() > 0, "Maximum allowed modules should be greater than 0");
}

#[test]
fn test_stake_operations() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test getting stake for an existing module
    let result = fixture.rpc.list_modules(netuid);
    if let Ok(modules) = result {
        if let Some(first_module) = modules.first() {
            let stake_result = fixture.rpc.get_stake(&first_module.name, netuid);
            assert!(stake_result.is_ok(), "Failed to get stake: {:?}", stake_result.err());
            let _stake = stake_result.unwrap();
        }
    }

    // Test getting stake for non-existent module
    let result = fixture.rpc.get_stake("non_existent_module", netuid);
    assert!(result.is_ok(), "Get stake should not error for non-existent module");
    assert_eq!(result.unwrap(), 0, "Non-existent module should have 0 stake");
}

#[test]
fn test_network_parameters() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test getting minimum stake
    let min_stake = fixture.rpc.get_min_stake(netuid);
    assert!(min_stake.is_ok(), "Failed to get minimum stake");
    let min_stake = min_stake.unwrap();
    assert!(min_stake > 0, "Minimum stake should be positive");

    // Test getting maximum allowed modules
    let max_modules = fixture.rpc.get_max_allowed_modules();
    assert!(max_modules.is_ok(), "Failed to get max allowed modules");
    let max_modules = max_modules.unwrap();
    assert!(max_modules > 0, "Maximum allowed modules should be positive");
    
    // Verify max modules is reasonable (between 100 and 10000)
    assert!(max_modules >= 100 && max_modules <= 10000, 
            "Max modules should be between 100 and 10000, got {}", max_modules);
}

#[test]
fn test_module_metadata() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test modules have valid metadata
    let result = fixture.rpc.list_modules(netuid);
    if let Ok(modules) = result {
        for module in modules {
            // Verify module name is not empty
            assert!(!module.name.is_empty(), "Module name should not be empty");
            
            // Verify address is valid SS58 format
            assert!(module.address.starts_with("5"), "Module address should be valid SS58");
            assert_eq!(module.address.len(), 48, "Module address should be 48 chars");

            // If metadata exists, verify it's valid UTF-8
            if let Some(metadata) = module.metadata {
                assert!(String::from_utf8(metadata.into_bytes()).is_ok(), 
                        "Module metadata should be valid UTF-8");
            }
        }
    }
}

#[test]
fn test_module_operations() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test listing modules
    let result = fixture.rpc.list_modules(netuid);
    assert!(result.is_ok(), "Failed to list modules: {:?}", result.err());
    let modules = result.unwrap();
    
    // If there are modules, test getting one
    if let Some(first_module) = modules.first() {
        let result = fixture.rpc.get_module(&first_module.name, netuid);
        assert!(result.is_ok(), "Failed to get module: {:?}", result.err());
        
        if let Ok(Some(module)) = result {
            assert_eq!(module.name, first_module.name);
            assert_eq!(module.address, first_module.address);
            
            // Test getting stake for existing module
            let stake_result = fixture.rpc.get_stake(&module.name, netuid);
            assert!(stake_result.is_ok(), "Failed to get stake: {:?}", stake_result.err());
            let _stake = stake_result.unwrap();
        }
    }

    // Test write operations - these should fail without a funded key
    let test_module = create_test_module("test_module");
    
    // Test registration
    let result = fixture.rpc.register_module(test_module.clone(), netuid);
    assert!(result.is_err(), "Registration should fail without funded key");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Permission denied: Requires funded key"),
           "Error should indicate funded key requirement: {}", err);

    // Test staking
    let result = fixture.rpc.stake(&test_module.name, 500000000000, netuid);
    assert!(result.is_err(), "Staking should fail without funded key");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Permission denied: Requires funded key"),
           "Error should indicate funded key requirement: {}", err);

    // Test unstaking
    let result = fixture.rpc.unstake(&test_module.name, 250000000000, netuid);
    assert!(result.is_err(), "Unstaking should fail without funded key");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Permission denied: Requires funded key"),
           "Error should indicate funded key requirement: {}", err);
}

#[test]
fn test_write_operations() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test module registration
    let module = create_test_module("test_write_module");
    let result = fixture.rpc.register_module(module.clone(), netuid);
    
    // Since we can't actually write to chain, verify error contains expected message
    assert!(result.is_err(), "Registration should fail without chain access");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("permission") || err.contains("access") || err.contains("unauthorized"),
            "Error should indicate permission/access issue");

    // Test stake operation
    let result = fixture.rpc.stake(&module.name, 1000, netuid);
    assert!(result.is_err(), "Stake should fail without chain access");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("permission") || err.contains("access") || err.contains("unauthorized"),
            "Error should indicate permission/access issue");
}

#[test]
fn test_invalid_network() {
    // Create a new RPC instance with invalid URL
    let rpc = CommuneRPC::new(
        "python3".to_string(),
        "wss://invalid-commune-node.example.com".to_string()
    );
    let netuid = 0;

    // Operations should fail with network error
    let result = rpc.get_module("test_module", netuid);
    assert!(result.is_err(), "Get module should fail with invalid network");

    let result = rpc.list_modules(netuid);
    assert!(result.is_err(), "List modules should fail with invalid network");
}

#[test]
fn test_mainnet_operations() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Test getting stake for an existing module
    let result = fixture.rpc.list_modules(netuid);
    if let Ok(modules) = result {
        if let Some(first_module) = modules.first() {
            let stake_result = fixture.rpc.get_stake(&first_module.name, netuid);
            assert!(stake_result.is_ok(), "Failed to get stake: {:?}", stake_result.err());
            let _stake = stake_result.unwrap();
            // Stake is always non-negative since it's u64
        }
    }

    // Test getting stake for non-existent module
    let result = fixture.rpc.get_stake("non_existent_module", netuid);
    assert!(result.is_ok(), "Get stake should not error for non-existent module");
    assert_eq!(result.unwrap(), 0, "Non-existent module should have 0 stake");
}

#[test]
fn test_testnet_operations() {
    let fixture = TestFixture::new(true);
    let netuid = 0;

    // Test basic network operations on testnet
    let modules = fixture.rpc.list_modules(netuid);
    assert!(modules.is_ok(), "Failed to list modules on testnet");

    // Test module registration (should fail without funded key)
    let module = create_test_module("test_module");
    let result = fixture.rpc.register_module(module, netuid);
    assert!(result.is_err(), "Module registration should fail without funded key");
    assert!(result.unwrap_err().to_string().contains("Permission denied"));
}

// Note: The following tests are skipped as they require write access to the network
#[test]
#[ignore]
fn test_register_and_stake() {
    let fixture = TestFixture::new(false);
    let netuid = 0;

    // Create and register a test module
    let module = create_test_module("test_module_1");
    let result = fixture.rpc.register_module(module.clone(), netuid);
    assert!(result.is_ok(), "Failed to register module: {:?}", result.err());

    // Add stake
    let stake_amount = 500;
    let result = fixture.rpc.stake("test_module_1", stake_amount, netuid);
    assert!(result.is_ok(), "Failed to add stake: {:?}", result.err());

    // Remove stake
    let result = fixture.rpc.unstake("test_module_1", stake_amount, netuid);
    assert!(result.is_ok(), "Failed to remove stake: {:?}", result.err());
}
