//! Registrar implementation for the Synapse Subnet project.
//! 
//! This crate provides the module registry and build system for managing
//! inference modules, supporting both Docker-based and native implementations.

pub mod interface;
mod module;
mod registry;

pub use interface::{
    Health, HealthStatus, InferenceModule, InferenceParameters,
    Input, MetricsData, ModuleCapabilities, Output, ResourceRequirements,
    TokenUsage,
};
pub use module::{Module, ModuleType};
pub use registry::{LocalRegistry, ModuleConfig, ModuleStatus, ModuleState, Registry, RegistryError};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
