use crate::{CreateModuleRequest, Module, ModuleStatus, ModuleType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use async_trait::async_trait;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Module already exists: {0}")]
    ModuleExists(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleResponse {
    name: String,
    status: ModuleStatus,
    #[serde(rename = "type")]
    module_type: ModuleType,
}

#[async_trait]
pub trait RegistrarClientTrait: Send + Sync {
    async fn register_module(&self, module: Module) -> Result<(), ClientError>;
    async fn start_module(&self, name: &str) -> Result<(), ClientError>;
    async fn unregister_module(&self, name: &str) -> Result<(), ClientError>;
    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError>;
    async fn update_module_status(&self, name: &str, status: ModuleStatus) -> Result<(), ClientError>;
    async fn register_miner(&self, uid: &str, key: &str, name: &str) -> Result<(), ClientError>;
    async fn list_modules(&self) -> Result<Vec<Module>, ClientError>;
    fn clone_box(&self) -> Box<dyn RegistrarClientTrait>;
}

#[derive(Debug, Clone)]
pub struct RegistrarClient {
    client: Client,
    base_url: String,
}

impl RegistrarClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl RegistrarClientTrait for RegistrarClient {
    async fn register_module(&self, module: Module) -> Result<(), ClientError> {
        let request = CreateModuleRequest {
            name: module.name.clone(),
            module_type: module.module_type,
        };

        let response = self
            .client
            .post(&format!("{}/modules", self.base_url))
            .json(&request)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::CREATED => Ok(()),
            reqwest::StatusCode::BAD_REQUEST => {
                let error = response.text().await?;
                Err(ClientError::BadRequest(error))
            }
            reqwest::StatusCode::CONFLICT => {
                Err(ClientError::ModuleExists(module.name))
            }
            _ => {
                let error = response.text().await?;
                Err(ClientError::Internal(error))
            }
        }
    }

    async fn unregister_module(&self, name: &str) -> Result<(), ClientError> {
        let response = self
            .client
            .delete(&format!("{}/modules/{}", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::NO_CONTENT => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.text().await?;
                Err(ClientError::Internal(error))
            }
        }
    }

    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError> {
        let response = self
            .client
            .get(&format!("{}/modules/{}/status", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let status = response.json::<ModuleStatus>().await?;
                Ok(status)
            }
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.text().await?;
                Err(ClientError::Internal(error))
            }
        }
    }

    async fn update_module_status(&self, name: &str, status: ModuleStatus) -> Result<(), ClientError> {
        let response = self
            .client
            .put(&format!("{}/modules/{}/status", self.base_url, name))
            .json(&status)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::NO_CONTENT => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.text().await?;
                Err(ClientError::Internal(error))
            }
        }
    }

    async fn start_module(&self, name: &str) -> Result<(), ClientError> {
        let response = self
            .client
            .post(&format!("{}/modules/{}/start", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::NO_CONTENT => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.text().await?;
                Err(ClientError::Internal(error))
            }
        }
    }

    async fn register_miner(&self, uid: &str, key: &str, name: &str) -> Result<(), ClientError> {
        // TODO: implement register_miner
        unimplemented!()
    }

    async fn list_modules(&self) -> Result<Vec<Module>, ClientError> {
        let url = format!("{}/modules", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(ClientError::Internal("Failed to list modules".to_string()));
        }
        
        let modules: Vec<ModuleResponse> = response.json().await?;
        Ok(modules.into_iter().map(|m| Module {
            name: m.name,
            status: m.status,
            module_type: m.module_type,
        }).collect())
    }

    fn clone_box(&self) -> Box<dyn RegistrarClientTrait> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_register_module() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/modules"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let client = RegistrarClient::new(mock_server.uri());

        let module = Module {
            name: "test".to_string(),
            module_type: ModuleType::Subnet,
            status: ModuleStatus::Stopped,
        };

        let result = client.register_module(module).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unregister_module() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/modules/test"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = RegistrarClient::new(mock_server.uri());

        let result = client.unregister_module("test").await;
        assert!(result.is_ok());
    }
}
