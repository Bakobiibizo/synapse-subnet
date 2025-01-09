//! Registrar implementation for the Synapse Subnet project.
//! 
//! This crate provides the module registry and build system for managing
//! inference modules, supporting both Docker-based and native implementations.

pub mod interface;
pub mod module;
pub mod registry;
pub mod docker;
pub mod verification;

pub use interface::{
    Health, HealthStatus, InferenceModule, InferenceParameters,
    Input, MetricsData, ModuleCapabilities, Output, ResourceRequirements,
    TokenUsage,
};
pub use module::{Module, ModuleType, ModuleStatus, ModuleState, ModuleRuntime};
pub use registry::{LocalRegistry, RegistryError};
pub use docker::DockerModuleRuntime;
pub use verification::{ModuleVerifier, VerificationConfig, VerificationError};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
