use std::time::Duration;
use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use async_trait::async_trait;
use crate::traits::RegistrarClientTrait;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub status: ModuleStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleStatus {
    pub state: ModuleState,
    pub health: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ModuleState {
    Created,
    Running,
    Stopped,
    Failed,
}

pub struct RegistrarClient {
    client: Client,
    base_url: String,
}

impl RegistrarClient {
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }
}

#[async_trait]
impl RegistrarClientTrait for RegistrarClient {
    async fn list_modules(&self) -> Result<Vec<Module>, ClientError> {
        let url = format!("{}/modules", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn get_module(&self, name: &str) -> Result<Module, ClientError> {
        let url = format!("{}/modules/{}", self.base_url, name);
        let response = self.client.get(&url).send().await?;
        
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::NOT_FOUND => Err(ClientError::ModuleNotFound(name.to_string())),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn create_module(&self, module: &Module) -> Result<i64, ClientError> {
        let url = format!("{}/modules", self.base_url);
        let response = self.client.post(&url)
            .json(module)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::CREATED => Ok(response.json().await?),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError> {
        let url = format!("{}/modules/{}/status", self.base_url, name);
        let response = self.client.get(&url).send().await?;
        
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::NOT_FOUND => Err(ClientError::ModuleNotFound(name.to_string())),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn update_module_status(&self, name: &str, status: &ModuleStatus) -> Result<(), ClientError> {
        let url = format!("{}/modules/{}/status", self.base_url, name);
        let response = self.client.put(&url)
            .json(status)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::NOT_FOUND => Err(ClientError::ModuleNotFound(name.to_string())),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn start_module(&self, name: &str) -> Result<(), ClientError> {
        let url = format!("{}/modules/{}/start", self.base_url, name);
        let response = self.client.post(&url).send().await?;
        
        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::NOT_FOUND => Err(ClientError::ModuleNotFound(name.to_string())),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn register_module(&self, module: &Module) -> Result<(), ClientError> {
        let url = format!("{}/modules", self.base_url);
        let response = self.client.post(&url)
            .json(module)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => Ok(()),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn unregister_module(&self, name: &str) -> Result<(), ClientError> {
        let url = format!("{}/modules/{}", self.base_url, name);
        let response = self.client.delete(&url).send().await?;
        
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(ClientError::ModuleNotFound(name.to_string())),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }

    async fn register_miner(&self, uid: &str, key: &str, name: &str) -> Result<(), ClientError> {
        let url = format!("{}/miners", self.base_url);
        let payload = serde_json::json!({
            "uid": uid,
            "key": key,
            "name": name
        });
        
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => Ok(()),
            _ => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", response.status()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;
    use mockito::mock;

    #[test]
    fn test_create_module() {
        let mut server = mockito::Server::new();
        let base_url = server.url();
        
        let module = Module {
            id: 0,
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            status: ModuleStatus {
                state: ModuleState::Created,
                health: None,
                error: None,
            },
        };

        let _m = mock("POST", "/modules")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":1}"#)
            .create();

        let client = RegistrarClient::new(&base_url).unwrap();
        let result = block_on(client.create_module(&module));
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
