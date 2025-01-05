//! Registrar implementation for the Synapse Subnet project.
//! 
//! This crate provides the module registry and build system for managing
//! inference modules, supporting both Docker-based and native implementations.

pub mod interface;
pub mod module;

pub use interface::{
    Health, HealthStatus, InferenceModule, InferenceParameters,
    Input, MetricsData, ModuleCapabilities, Output, ResourceRequirements,
    TokenUsage,
};
pub use module::{Module, ModuleType};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
