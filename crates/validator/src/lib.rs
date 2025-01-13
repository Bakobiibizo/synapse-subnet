use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use docker_manager::{ContainerManager, DockerError};
use registrar::module::Module;
use registrar_api::client::{RegistrarClientTrait, ClientError as RegistrarError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use std::any::Any;

pub mod api;

/// Chain API trait for querying blockchain data
#[async_trait]
pub trait ChainApi: Send + Sync + 'static {
    async fn query_map(&self, field: &str) -> Result<Vec<String>, ValidatorError>;
    async fn query_map_for_miner(&self, uid: &str, field: &str) -> Result<String, ValidatorError>;
}

/// Response logger trait for tracking miner interactions
#[async_trait]
pub trait ResponseLoggerSync: Send + Sync + 'static {
    async fn log_request(&self, miner_uid: &str) -> Result<(), ValidatorError>;
    async fn log_response(&self, miner_uid: &str, success: bool) -> Result<(), ValidatorError>;
}

/// Async response logger trait for querying response data
#[async_trait]
pub trait ResponseLoggerAsync: Send + Sync + 'static {
    async fn get_success_rate(&self, miner_uid: &str) -> Result<f64, ValidatorError>;
    async fn get_last_response(&self, miner_uid: &str) -> Result<u64, ValidatorError>;
}

/// Default response logger implementation
#[derive(Default)]
pub struct DefaultResponseLogger {
    requests: RwLock<HashMap<String, Vec<u64>>>,
    responses: RwLock<HashMap<String, Vec<(u64, bool)>>>,
}

impl DefaultResponseLogger {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ResponseLoggerSync for DefaultResponseLogger {
    async fn log_request(&self, miner_uid: &str) -> Result<(), ValidatorError> {
        let mut requests = self.requests.write().await;
        let miner_requests = requests.entry(miner_uid.to_string()).or_insert_with(Vec::new);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        miner_requests.push(timestamp);
        Ok(())
    }

    async fn log_response(&self, miner_uid: &str, success: bool) -> Result<(), ValidatorError> {
        let mut responses = self.responses.write().await;
        let miner_responses = responses.entry(miner_uid.to_string()).or_insert_with(Vec::new);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        miner_responses.push((timestamp, success));
        Ok(())
    }
}

#[async_trait]
impl ResponseLoggerAsync for DefaultResponseLogger {
    async fn get_success_rate(&self, miner_uid: &str) -> Result<f64, ValidatorError> {
        let responses = self.responses.read().await;
        if let Some(miner_responses) = responses.get(miner_uid) {
            if miner_responses.is_empty() {
                return Ok(0.0);
            }
            let success_count = miner_responses.iter().filter(|(_, success)| *success).count();
            Ok(success_count as f64 / miner_responses.len() as f64)
        } else {
            Ok(0.0)
        }
    }

    async fn get_last_response(&self, miner_uid: &str) -> Result<u64, ValidatorError> {
        let responses = self.responses.read().await;
        if let Some(miner_responses) = responses.get(miner_uid) {
            if let Some((timestamp, _)) = miner_responses.last() {
                return Ok(*timestamp);
            }
        }
        Ok(0)
    }
}

/// Response tracker for monitoring miner responses
pub struct ResponseTracker {
    success_rates: RwLock<HashMap<String, f64>>,
    last_responses: RwLock<HashMap<String, u64>>,
}

impl ResponseTracker {
    pub fn new() -> Self {
        Self {
            success_rates: RwLock::new(HashMap::new()),
            last_responses: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update_stats(&self, miner_uid: &str, success: bool, timestamp: u64) {
        let mut success_rates = self.success_rates.write().await;
        let mut last_responses = self.last_responses.write().await;

        // Update success rate
        let rate = success_rates.entry(miner_uid.to_string()).or_insert(0.0);
        *rate = (*rate + if success { 1.0 } else { 0.0 }) / 2.0;

        // Update last response
        last_responses.insert(miner_uid.to_string(), timestamp);
    }
}

/// Represents the monitoring status of a miner
#[derive(Debug, Serialize, Deserialize)]
pub struct MinerStatus {
    pub uid: String,
    pub key: String,
    pub name: String,
    pub address: String,
    pub emissions: u64,
    pub incentives: u64,
    pub dividens: u64,
    pub stakefrom: u64,
    pub in_immunity: bool,
    pub active_status: bool,
    pub last_response: u64,
    pub score: f64,
}

impl MinerStatus {
    fn new(
        uid: String,
        key: String,
        name: String,
        address: String,
        emissions: u64,
        incentives: u64,
        dividens: u64,
        stakefrom: u64,
        in_immunity: bool,
        last_response: u64,
        score: f64,
    ) -> Self {
        let active_status = emissions > 0 || incentives > 0 || dividens > 0;
        
        Self {
            uid,
            key,
            name,
            address,
            emissions,
            incentives,
            dividens,
            stakefrom,
            in_immunity,
            active_status,
            last_response,
            score,
        }
    }
}

/// Errors that can occur during validator operations
#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("Docker error: {0}")]
    Docker(#[from] DockerError),
    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistrarError),
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Invalid module: {0}")]
    InvalidModule(String),
    #[error("Miner not found: {0}")]
    MinerNotFound(String),
    #[error("No responses recorded for miner: {0}")]
    NoResponses(String),
    #[error("API communication error: {0}")]
    ApiCommunication(String),
}

/// Module configuration for subnet validators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub image: String,
    pub tag: String,
    pub port: u16,
    pub env: Option<HashMap<String, String>>,
    pub volumes: Option<HashMap<String, String>>,
}

/// Subnet module trait for injecting response logging
pub trait SubnetModule: Clone + Any + Send + Sync + 'static {
    fn set_response_logger(&mut self, logger: Arc<dyn ResponseLoggerSync>);
}

/// Subnet injector for adding response logging to subnet modules
pub struct SubnetInjector {
    logger: Arc<dyn ResponseLoggerSync>,
}

impl SubnetInjector {
    pub fn new(logger: Arc<dyn ResponseLoggerSync>) -> Self {
        Self { logger }
    }

    pub fn inject_logger<T>(&self, mut subnet_module: T) -> T 
    where 
        T: SubnetModule
    {
        subnet_module.set_response_logger(self.logger.clone());
        subnet_module
    }
}

/// Manages the validator's modules and subnet configurations
pub struct ValidatorManager {
    #[allow(dead_code)]
    docker: Arc<dyn ContainerManager>,
    registrar: Box<dyn RegistrarClientTrait>,
    subnet_modules: RwLock<HashMap<String, Module>>,
    #[allow(dead_code)]
    module_configs: RwLock<HashMap<String, ModuleConfig>>,
    response_logger: Arc<DefaultResponseLogger>,
    #[allow(dead_code)]
    response_tracker: Arc<ResponseTracker>,
    #[allow(dead_code)]
    subnet_injector: SubnetInjector,
}

impl ValidatorManager {
    pub fn new(docker: Arc<dyn ContainerManager>, registrar: Box<dyn RegistrarClientTrait>) -> Self {
        let response_logger = Arc::new(DefaultResponseLogger::new());
        let subnet_injector = SubnetInjector::new(response_logger.clone());
        let response_tracker = Arc::new(ResponseTracker::new());
        
        Self {
            docker,
            registrar,
            subnet_modules: RwLock::new(HashMap::new()),
            module_configs: RwLock::new(HashMap::new()),
            response_logger,
            response_tracker,
            subnet_injector,
        }
    }

    pub async fn start_module(&self, name: &str) -> Result<(), ValidatorError> {
        // Start module with registrar
        self.registrar.start_module(name).await?;

        Ok(())
    }

    pub async fn register_module(&self, module: Module) -> Result<(), ValidatorError> {
        // Register with registrar first
        self.registrar.register_module(module.clone()).await?;

        // Store module
        let mut modules = self.subnet_modules.write().await;
        let module = modules
            .entry(module.name.clone())
            .or_insert(module);

        // Initialize module
        self.start_module(&module.name).await?;

        Ok(())
    }

    pub async fn unregister_module(&self, name: &str) -> Result<(), ValidatorError> {
        // Unregister with registrar first
        self.registrar.unregister_module(name).await?;

        // Remove from storage
        let mut modules = self.subnet_modules.write().await;
        modules.remove(name);

        Ok(())
    }

    pub async fn get_monitoring_status(&self, _chain_api: Arc<dyn ChainApi>) -> Result<Vec<MinerStatus>, ValidatorError> {
        let mut statuses = Vec::new();

        // Get modules from registrar
        let modules = self.registrar.list_modules().await.map_err(|e| ValidatorError::RegistryError(e))?;
        
        for module in modules {
            let status = MinerStatus::new(
                module.name.clone(),
                "default_key".to_string(),
                module.name.clone(),
                "0x0".to_string(),
                100,
                50,
                25,
                1000,
                false,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                0.0,
            );
            statuses.push(status);
        }

        Ok(statuses)
    }

    pub async fn register_miner(&self, uid: &str, key: &str, name: &str) -> Result<(), ValidatorError> {
        // Register with registrar first
        self.registrar.register_miner(uid, key, name).await?;

        Ok(())
    }

    pub async fn log_miner_response(&self, miner_uid: &str, success: bool) -> Result<(), ValidatorError> {
        self.response_logger.log_response(miner_uid, success).await?;
        Ok(())
    }

    pub async fn get_modules(&self) -> Result<Vec<Module>, ValidatorError> {
        let modules = self.subnet_modules.read().await;
        Ok(modules.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use mockall::automock;
    use wiremock::MockServer;
    use tower::ServiceExt;
    use axum::body::{Body, to_bytes};
    use hyper::Request;
    use docker_manager::{ContainerStatus, ContainerState};
    use registrar::module::ModuleStatus;

    // Mock registrar client for testing
    #[derive(Clone)]
    struct MockRegistrarClient {
        base_url: String,
    }

    impl MockRegistrarClient {
        fn new(base_url: String) -> Self {
            Self { base_url }
        }
    }

    #[async_trait]
    impl RegistrarClientTrait for MockRegistrarClient {
        async fn list_modules(&self) -> Result<Vec<Module>, RegistrarError> {
            let module = Module {
                name: "miner1".to_string(),
                module_type: registrar::module::ModuleType::Docker {
                    image: "test".to_string(),
                    tag: "latest".to_string(),
                    port: 8080,
                    env: None,
                    volumes: None,
                    health_check: None,
                },
                status: ModuleStatus::new(),
            };
            Ok(vec![module])
        }

        async fn register_module(&self, _module: Module) -> Result<(), RegistrarError> {
            Ok(())
        }

        async fn start_module(&self, _name: &str) -> Result<(), RegistrarError> {
            Ok(())
        }

        async fn unregister_module(&self, _name: &str) -> Result<(), RegistrarError> {
            Ok(())
        }

        async fn get_module_status(&self, _name: &str) -> Result<ModuleStatus, RegistrarError> {
            Ok(ModuleStatus::new())
        }

        async fn update_module_status(&self, _name: &str, _status: ModuleStatus) -> Result<(), RegistrarError> {
            Ok(())
        }

        async fn register_miner(&self, _uid: &str, _key: &str, _name: &str) -> Result<(), RegistrarError> {
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn RegistrarClientTrait> {
            Box::new(Self::new(self.base_url.clone()))
        }
    }

    #[automock]
    pub trait DockerManager: Send + Sync {
        fn start_container(&self, name: &str) -> Result<(), DockerError>;
        fn stop_container(&self, name: &str) -> Result<(), DockerError>;
    }

    #[async_trait]
    impl ContainerManager for MockDockerManager {
        async fn create_container(&self, _config: docker_manager::ContainerConfig) -> Result<(), DockerError> {
            Ok(())
        }

        async fn remove_container(&self, _name: &str) -> Result<(), DockerError> {
            Ok(())
        }

        async fn start_container(&self, _name: &str) -> Result<(), DockerError> {
            Ok(())
        }

        async fn stop_container(&self, _name: &str) -> Result<(), DockerError> {
            Ok(())
        }

        async fn get_container_status(&self, _name: &str) -> Result<ContainerStatus, DockerError> {
            Ok(ContainerStatus {
                state: ContainerState::Running,
                health: Some("healthy".to_string()),
                exit_code: None,
                error: None,
            })
        }

        async fn list_containers(&self) -> Result<Vec<ContainerStatus>, DockerError> {
            Ok(Vec::new())
        }
    }

    struct MockChainApi;

    impl MockChainApi {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ChainApi for MockChainApi {
        async fn query_map(&self, _field: &str) -> Result<Vec<String>, ValidatorError> {
            Ok(vec!["test".to_string()])
        }

        async fn query_map_for_miner(&self, _uid: &str, _field: &str) -> Result<String, ValidatorError> {
            Ok("test".to_string())
        }
    }

    #[tokio::test]
    async fn test_response_logging() {
        let docker = Arc::new(MockDockerManager::new());
        let registrar = Box::new(MockRegistrarClient::new("http://localhost:8080".to_string()));
        let validator = ValidatorManager::new(docker, registrar);

        validator.log_miner_response("miner1", true).await.unwrap();
        let success_rate = validator.response_logger.get_success_rate("miner1").await.unwrap();
        assert_eq!(success_rate, 1.0);

        validator.log_miner_response("miner1", false).await.unwrap();
        let success_rate = validator.response_logger.get_success_rate("miner1").await.unwrap();
        assert_eq!(success_rate, 0.5);
    }

    #[tokio::test]
    async fn test_monitoring_endpoint() {
        let docker = Arc::new(MockDockerManager::new());
        let registrar = Box::new(MockRegistrarClient::new("http://localhost:8080".to_string()));
        let validator = ValidatorManager::new(docker, registrar);
        let chain_api = Arc::new(MockChainApi::new());

        let status = validator.get_monitoring_status(chain_api).await.unwrap();
        assert!(!status.is_empty());
        
        let miner_status = &status[0];
        assert_eq!(miner_status.uid, "miner1");
        assert_eq!(miner_status.key, "default_key");
        assert_eq!(miner_status.name, "miner1");
        assert_eq!(miner_status.address, "0x0");
        assert!(miner_status.active_status);
    }
}

/// API state for handling requests
#[derive(Clone)]
pub struct ApiState {
    pub validator: Arc<ValidatorManager>,
    pub chain_api: Arc<dyn ChainApi>,
}
