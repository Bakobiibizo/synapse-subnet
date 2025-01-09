use crate::{CreateModuleRequest, Module, ModuleStatus, ModuleType};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

    pub async fn register_module(&self, module: Module) -> Result<(), ClientError> {
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
            reqwest::StatusCode::CONFLICT => {
                Err(ClientError::ModuleExists(module.name))
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::BadRequest(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
            _ => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::Internal(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
        }
    }

    pub async fn unregister_module(&self, name: &str) -> Result<(), ClientError> {
        let response = self
            .client
            .delete(&format!("{}/modules/{}", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::Internal(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
        }
    }

    pub async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError> {
        let response = self
            .client
            .get(&format!("{}/modules/{}/status", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                Ok(response.json::<ModuleStatus>().await?)
            }
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::Internal(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
        }
    }

    pub async fn update_module_status(&self, name: &str, status: ModuleStatus) -> Result<(), ClientError> {
        let response = self
            .client
            .put(&format!("{}/modules/{}/status", self.base_url, name))
            .json(&status)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::Internal(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
        }
    }

    pub async fn start_module(&self, name: &str) -> Result<(), ClientError> {
        let response = self
            .client
            .post(&format!("{}/modules/{}/start", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            }
            _ => {
                let error = response.json::<serde_json::Value>().await?;
                Err(ClientError::Internal(error["error"].as_str().unwrap_or("Unknown error").to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_register_module() {
        let mock_server = MockServer::start().await;
        let client = RegistrarClient::new(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/modules"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let module = Module {
            name: "test-module".to_string(),
            module_type: ModuleType::Docker {
                image: "test".to_string(),
                tag: "latest".to_string(),
                port: 8080,
                env: None,
                volumes: None,
                health_check: None,
            },
            status: ModuleStatus::new(),
        };

        let result = client.register_module(module).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unregister_module() {
        let mock_server = MockServer::start().await;
        let client = RegistrarClient::new(mock_server.uri());

        Mock::given(method("DELETE"))
            .and(path("/modules/test-module"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let result = client.unregister_module("test-module").await;
        assert!(result.is_ok());
    }
}
