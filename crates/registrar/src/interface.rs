use async_trait::async_trait;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[pymethods]
impl HealthStatus {
    fn __str__(&self) -> String {
        match self {
            HealthStatus::Healthy => "Healthy".to_string(),
            HealthStatus::Degraded => "Degraded".to_string(),
            HealthStatus::Unhealthy => "Unhealthy".to_string(),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    #[pyo3(get, set)]
    pub status: HealthStatus,
    #[pyo3(get, set)]
    pub message: Option<String>,
}

#[pymethods]
impl Health {
    #[new]
    pub fn new(status: HealthStatus, message: Option<String>) -> Self {
        Self { status, message }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleCapabilities {
    #[pyo3(get, set)]
    pub supported_models: Vec<String>,
    #[pyo3(get, set)]
    pub max_batch_size: usize,
    #[pyo3(get, set)]
    pub max_sequence_length: usize,
    #[pyo3(get, set)]
    pub supported_precisions: Vec<String>,
}

#[pymethods]
impl ModuleCapabilities {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParameters {
    #[pyo3(get, set)]
    pub temperature: f32,
    #[pyo3(get, set)]
    pub top_p: f32,
    #[pyo3(get, set)]
    pub max_tokens: usize,
}

#[pymethods]
impl InferenceParameters {
    #[new]
    pub fn new(temperature: f32, top_p: f32, max_tokens: usize) -> Self {
        Self {
            temperature,
            top_p,
            max_tokens,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    #[pyo3(get, set)]
    pub text: String,
    #[pyo3(get, set)]
    pub parameters: InferenceParameters,
}

#[pymethods]
impl Input {
    #[new]
    pub fn new(text: String, parameters: InferenceParameters) -> Self {
        Self { text, parameters }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    #[pyo3(get, set)]
    pub prompt_tokens: usize,
    #[pyo3(get, set)]
    pub completion_tokens: usize,
    #[pyo3(get, set)]
    pub total_tokens: usize,
}

#[pymethods]
impl TokenUsage {
    #[new]
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    #[pyo3(get, set)]
    pub text: String,
    #[pyo3(get, set)]
    pub usage: TokenUsage,
}

#[pymethods]
impl Output {
    #[new]
    pub fn new(text: String, usage: TokenUsage) -> Self {
        Self { text, usage }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceRequirements {
    #[pyo3(get, set)]
    pub cpu_cores: f32,
    #[pyo3(get, set)]
    pub memory_mb: usize,
    #[pyo3(get, set)]
    pub gpu_memory_mb: Option<usize>,
}

#[pymethods]
impl ResourceRequirements {
    #[new]
    pub fn new(cpu_cores: f32, memory_mb: usize, gpu_memory_mb: Option<usize>) -> Self {
        Self {
            cpu_cores,
            memory_mb,
            gpu_memory_mb,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsData {
    #[pyo3(get, set)]
    pub resource_usage: ResourceRequirements,
    #[pyo3(get, set)]
    pub inference_count: usize,
    #[pyo3(get, set)]
    pub average_latency_ms: f64,
    #[pyo3(get, set)]
    pub error_count: usize,
}

#[pymethods]
impl MetricsData {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
pub trait InferenceModule: Send + Sync {
    /// Initialize the module
    async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Check the health of the module
    async fn health_check(&self) -> Result<Health, Box<dyn Error + Send + Sync>>;
    
    /// Get module capabilities
    fn get_capabilities(&self) -> ModuleCapabilities;
    
    /// Run inference on input
    async fn run_inference(&self, input: Input) -> Result<Output, Box<dyn Error + Send + Sync>>;
    
    /// Get module metrics
    fn get_metrics(&self) -> MetricsData;
}
