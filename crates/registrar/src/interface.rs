use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Represents the health status of a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub status: HealthStatus,
    pub message: String,
}

/// Possible health status values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Module capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCapabilities {
    pub name: String,
    pub version: String,
    pub model_type: String,
    pub max_batch_size: usize,
    pub max_sequence_length: usize,
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for the module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_memory_mb: usize,
    pub min_cpu_cores: f32,
    pub gpu_required: bool,
    pub min_gpu_memory_mb: Option<usize>,
}

/// Input for inference requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub text: String,
    pub parameters: InferenceParameters,
}

/// Parameters for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParameters {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
}

/// Output from inference requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub text: String,
    pub usage: TokenUsage,
}

/// Token usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Metrics data for module monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub requests_processed: usize,
    pub average_latency_ms: f64,
    pub error_count: usize,
    pub memory_usage_mb: usize,
}

/// Base trait for inference modules
#[async_trait]
pub trait InferenceModule: Send + Sync {
    /// Initialize the module
    async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Check module health
    async fn health_check(&self) -> Result<Health, Box<dyn Error + Send + Sync>>;
    
    /// Get module capabilities
    fn get_capabilities(&self) -> ModuleCapabilities;
    
    /// Run inference
    async fn run_inference(&self, input: Input) -> Result<Output, Box<dyn Error + Send + Sync>>;
    
    /// Get module metrics
    fn get_metrics(&self) -> MetricsData;
}
