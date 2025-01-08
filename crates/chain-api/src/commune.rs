use std::error::Error;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json;

/// Represents a Commune AI module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommuneModule {
    pub name: String,
    pub address: String,
    pub stake: u64,
    pub metadata: Option<String>,
}

/// Interface for interacting with Commune AI network
pub trait CommuneInterface {
    /// Register a new module on the network
    fn register_module(&self, module: CommuneModule, netuid: u16) -> Result<(), Box<dyn Error>>;
    
    /// Get module information by name
    fn get_module(&self, name: &str, netuid: u16) -> Result<Option<CommuneModule>, Box<dyn Error>>;
    
    /// List all registered modules in a subnet
    fn list_modules(&self, netuid: u16) -> Result<Vec<CommuneModule>, Box<dyn Error>>;
    
    /// Stake tokens on a module
    fn stake(&self, module_name: &str, amount: u64, netuid: u16) -> Result<(), Box<dyn Error>>;
    
    /// Unstake tokens from a module
    fn unstake(&self, module_name: &str, amount: u64, netuid: u16) -> Result<(), Box<dyn Error>>;
    
    /// Get current stake for a module
    fn get_stake(&self, module_name: &str, netuid: u16) -> Result<u64, Box<dyn Error>>;

    /// Get minimum stake required for a subnet
    fn get_min_stake(&self, netuid: u16) -> Result<u64, Box<dyn Error>>;

    /// Get maximum allowed modules in a subnet
    fn get_max_allowed_modules(&self) -> Result<u64, Box<dyn Error>>;
}

/// Implementation of CommuneInterface that uses Python communex library via subprocess
pub struct CommuneRPC {
    python_path: String,
    rpc_url: String,
}

impl CommuneRPC {
    pub fn new(python_path: String, rpc_url: String) -> Self {
        Self { 
            python_path,
            rpc_url,
        }
    }

    fn run_python_command(&self, script_args: &[&str]) -> Result<String, Box<dyn Error>> {
        let output = Command::new(&self.python_path)
            .arg("commune_rpc.py")
            .args(script_args)
            .env("COMMUNE_RPC_URL", &self.rpc_url)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.into());
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}

impl CommuneInterface for CommuneRPC {
    fn register_module(&self, module: CommuneModule, netuid: u16) -> Result<(), Box<dyn Error>> {
        let module_json = serde_json::to_string(&module)?;
        self.run_python_command(&["register_module", &module_json, &netuid.to_string()])?;
        Ok(())
    }

    fn get_module(&self, name: &str, netuid: u16) -> Result<Option<CommuneModule>, Box<dyn Error>> {
        let result = self.run_python_command(&["get_module", name, &netuid.to_string()])?;
        if result.trim().is_empty() {
            return Ok(None);
        }
        let module = serde_json::from_str(&result)?;
        Ok(Some(module))
    }

    fn list_modules(&self, netuid: u16) -> Result<Vec<CommuneModule>, Box<dyn Error>> {
        let result = self.run_python_command(&["list_modules", &netuid.to_string()])?;
        let modules = serde_json::from_str(&result)?;
        Ok(modules)
    }

    fn stake(&self, module_name: &str, amount: u64, netuid: u16) -> Result<(), Box<dyn Error>> {
        self.run_python_command(&["stake", module_name, &amount.to_string(), &netuid.to_string()])?;
        Ok(())
    }

    fn unstake(&self, module_name: &str, amount: u64, netuid: u16) -> Result<(), Box<dyn Error>> {
        self.run_python_command(&["unstake", module_name, &amount.to_string(), &netuid.to_string()])?;
        Ok(())
    }

    fn get_stake(&self, module_name: &str, netuid: u16) -> Result<u64, Box<dyn Error>> {
        let result = self.run_python_command(&["get_stake", module_name, &netuid.to_string()])?;
        let stake = result.trim().parse()?;
        Ok(stake)
    }

    fn get_min_stake(&self, netuid: u16) -> Result<u64, Box<dyn Error>> {
        let result = self.run_python_command(&["get_min_stake", &netuid.to_string()])?;
        let min_stake = result.trim().parse()?;
        Ok(min_stake)
    }

    fn get_max_allowed_modules(&self) -> Result<u64, Box<dyn Error>> {
        let result = self.run_python_command(&["get_max_allowed_modules"])?;
        let max_modules = result.trim().parse()?;
        Ok(max_modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_environment() -> (CommuneRPC, PathBuf) {
        let test_dir = env::temp_dir().join("commune_rpc_tests");
        fs::create_dir_all(&test_dir).unwrap();
        
        let python_script = test_dir.join("commune_rpc.py");
        fs::copy(
            env::current_dir().unwrap().join("src/commune_rpc.py"),
            &python_script,
        ).unwrap();

        let rpc = CommuneRPC::new(
            "python3".to_string(),
            "http://localhost:9944".to_string(),
        );

        (rpc, test_dir)
    }

    fn cleanup_test_environment(test_dir: PathBuf) {
        fs::remove_dir_all(test_dir).unwrap_or(());
    }

    #[test]
    fn test_get_max_allowed_modules() {
        let (rpc, test_dir) = setup_test_environment();
        
        match rpc.get_max_allowed_modules() {
            Ok(max_modules) => {
                assert!(max_modules > 0, "Maximum allowed modules should be positive");
            }
            Err(e) => panic!("Failed to get max allowed modules: {}", e),
        }

        cleanup_test_environment(test_dir);
    }

    #[test]
    fn test_get_min_stake() {
        let (rpc, test_dir) = setup_test_environment();
        let netuid = 0;

        match rpc.get_min_stake(netuid) {
            Ok(min_stake) => {
                assert!(min_stake > 0, "Minimum stake should be positive");
            }
            Err(e) => panic!("Failed to get min stake: {}", e),
        }

        cleanup_test_environment(test_dir);
    }

    #[test]
    fn test_module_operations() {
        let (rpc, test_dir) = setup_test_environment();
        let netuid = 0;

        // Create a test module
        let module = CommuneModule {
            name: "test_module".to_string(),
            address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            stake: 1000000,
            metadata: Some("Test metadata".to_string()),
        };

        // Test registration
        match rpc.register_module(module.clone(), netuid) {
            Ok(()) => (),
            Err(e) => panic!("Failed to register module: {}", e),
        }

        // Test retrieval
        match rpc.get_module(&module.name, netuid) {
            Ok(Some(retrieved)) => {
                assert_eq!(retrieved.name, module.name);
                assert_eq!(retrieved.address, module.address);
            }
            Ok(None) => panic!("Module not found after registration"),
            Err(e) => panic!("Failed to get module: {}", e),
        }

        // Test listing
        match rpc.list_modules(netuid) {
            Ok(modules) => {
                assert!(!modules.is_empty(), "Module list should not be empty");
                assert!(modules.iter().any(|m| m.name == module.name));
            }
            Err(e) => panic!("Failed to list modules: {}", e),
        }

        // Test staking
        match rpc.stake(&module.name, 500000, netuid) {
            Ok(()) => (),
            Err(e) => panic!("Failed to stake: {}", e),
        }

        // Test getting stake
        match rpc.get_stake(&module.name, netuid) {
            Ok(stake) => {
                assert!(stake >= 500000, "Stake should be at least the amount we staked");
            }
            Err(e) => panic!("Failed to get stake: {}", e),
        }

        // Test unstaking
        match rpc.unstake(&module.name, 500000, netuid) {
            Ok(()) => (),
            Err(e) => panic!("Failed to unstake: {}", e),
        }

        cleanup_test_environment(test_dir);
    }
}
