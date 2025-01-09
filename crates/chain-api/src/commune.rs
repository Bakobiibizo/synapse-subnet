use std::process::Command;
use serde::{Deserialize, Serialize};
use std::error::Error;

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

/// Default mainnet URL for Commune network
pub const MAINNET_URL: &str = "wss://commune-api-node-2.communeai.net";
/// Testnet URL for Commune network
pub const TESTNET_URL: &str = "wss://testnet.api.communeai.net";

/// Implementation of CommuneInterface that uses Python communex library via subprocess
pub struct CommuneRPC {
    python_path: String,
    rpc_url: String,
}

impl CommuneRPC {
    pub fn new(python_path: String, url: String) -> Self {
        Self { 
            python_path,
            rpc_url: url,
        }
    }

    /// Create a new CommuneRPC instance using mainnet
    pub fn mainnet(python_path: String) -> Self {
        Self::new(python_path, MAINNET_URL.to_string())
    }

    /// Create a new CommuneRPC instance using testnet
    pub fn testnet(python_path: String) -> Self {
        Self::new(python_path, TESTNET_URL.to_string())
    }

    fn run_python_command(&self, script_args: &[&str]) -> Result<String, Box<dyn Error>> {
        // Set environment variables
        std::env::set_var("COMMUNE_RPC_URL", &self.rpc_url);

        // Build command with script path and args
        let mut cmd = Command::new("python3");
        cmd.arg(&self.python_path);
        cmd.args(script_args);
        
        // Execute command and capture output
        let output = cmd.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.into());
        }

        // Get stdout as string
        let stdout = String::from_utf8(output.stdout)?;
        let stdout = stdout.trim();
        
        if stdout.is_empty() {
            return Err("Empty response from Python script".into());
        }
        
        Ok(stdout.to_string())
    }
}

impl CommuneInterface for CommuneRPC {
    fn register_module(&self, module: CommuneModule, netuid: u16) -> Result<(), Box<dyn Error>> {
        let module_json = serde_json::to_string(&module)?;
        self.run_python_command(&["register_module", &module_json, &netuid.to_string()])?;
        Ok(())
    }

    fn get_module(&self, name: &str, netuid: u16) -> Result<Option<CommuneModule>, Box<dyn Error>> {
        let output = self.run_python_command(&["get_module", name, &netuid.to_string()])?;
        if output == "null" {
            return Ok(None);
        }
        let module: CommuneModule = serde_json::from_str(&output)?;
        Ok(Some(module))
    }

    fn list_modules(&self, netuid: u16) -> Result<Vec<CommuneModule>, Box<dyn Error>> {
        let output = self.run_python_command(&["list_modules", &netuid.to_string()])?;
        let modules: Vec<CommuneModule> = serde_json::from_str(&output)?;
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
        let output = self.run_python_command(&["get_stake", module_name, &netuid.to_string()])?;
        let stake: u64 = output.trim().parse()?;
        Ok(stake)
    }

    fn get_min_stake(&self, netuid: u16) -> Result<u64, Box<dyn Error>> {
        let output = self.run_python_command(&["get_min_stake", &netuid.to_string()])?;
        let min_stake: u64 = output.trim().parse()?;
        Ok(min_stake)
    }

    fn get_max_allowed_modules(&self) -> Result<u64, Box<dyn Error>> {
        let output = self.run_python_command(&["get_max_allowed_modules"])?;
        let max_modules: u64 = output.trim().parse()?;
        Ok(max_modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_environment() -> (CommuneRPC, std::path::PathBuf) {
        // Use mainnet URL for tests
        let mainnet_url = "wss://commune-api-node-2.communeai.net";
        env::set_var("COMMUNE_RPC_URL", mainnet_url);

        // Get path to Python script
        let current_dir = env::current_dir().unwrap();
        let script_path = current_dir.join("src").join("commune_rpc.py");
        
        let rpc = CommuneRPC::new(
            script_path.to_str().unwrap().to_string(),
            mainnet_url.to_string(),
        );

        (rpc, current_dir)
    }

    fn cleanup_test_environment(_test_dir: std::path::PathBuf) {
        // Clean up any test artifacts if needed
    }

    #[test]
    fn test_get_max_allowed_modules() {
        let (rpc, test_dir) = setup_test_environment();
        let result = rpc.get_max_allowed_modules();
        cleanup_test_environment(test_dir);

        assert!(result.is_ok(), "Failed to get max allowed modules: {:?}", result.err());
        let max_modules = result.unwrap();
        assert!(max_modules > 0, "Maximum allowed modules should be greater than 0");
    }

    #[test]
    fn test_get_min_stake() {
        let (rpc, test_dir) = setup_test_environment();
        let netuid = 0;
        let result = rpc.get_min_stake(netuid);
        cleanup_test_environment(test_dir);

        assert!(result.is_ok(), "Failed to get min stake: {:?}", result.err());
        let min_stake = result.unwrap();
        assert!(min_stake > 0, "Minimum stake should be greater than 0");
    }

    #[test]
    fn test_module_operations() {
        let (rpc, test_dir) = setup_test_environment();
        let netuid = 0;

        // Test module registration
        let test_module = CommuneModule {
            name: "test_module".to_string(),
            address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            stake: 1000,
            metadata: Some("Test module".to_string()),
        };

        let result = rpc.register_module(test_module.clone(), netuid);
        assert!(result.is_ok(), "Failed to register module: {:?}", result.err());

        // Test getting registered module
        let result = rpc.get_module(&test_module.name, netuid);
        assert!(result.is_ok(), "Failed to get module: {:?}", result.err());
        
        if let Ok(Some(module)) = result {
            assert_eq!(module.name, test_module.name);
            assert_eq!(module.address, test_module.address);
        }

        // Test listing modules
        let result = rpc.list_modules(netuid);
        assert!(result.is_ok(), "Failed to list modules: {:?}", result.err());
        
        // Test staking
        let result = rpc.stake(&test_module.name, 500, netuid);
        assert!(result.is_ok(), "Failed to stake: {:?}", result.err());

        // Test getting stake
        let result = rpc.get_stake(&test_module.name, netuid);
        assert!(result.is_ok(), "Failed to get stake: {:?}", result.err());
        
        if let Ok(stake) = result {
            assert!(stake >= 500, "Stake should be at least 500");
        }

        // Test unstaking
        let result = rpc.unstake(&test_module.name, 250, netuid);
        assert!(result.is_ok(), "Failed to unstake: {:?}", result.err());

        cleanup_test_environment(test_dir);
    }
}
