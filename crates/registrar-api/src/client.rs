use std::time::Duration;
use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use thiserror::Error;

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

    pub async fn list_modules(&self) -> Result<Vec<Module>, ClientError> {
        let response = self.client
            .get(&format!("{}/modules", self.base_url))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let modules = response.json().await?;
                Ok(modules)
            },
            status => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", status
            ))),
        }
    }

    pub async fn get_module(&self, name: &str) -> Result<Module, ClientError> {
        let response = self.client
            .get(&format!("{}/modules/{}", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let module = response.json().await?;
                Ok(module)
            },
            StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            },
            status => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", status
            ))),
        }
    }

    pub async fn create_module(&self, module: &Module) -> Result<i64, ClientError> {
        let response = self.client
            .post(&format!("{}/modules", self.base_url))
            .json(&module)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let id = response.json().await?;
                Ok(id)
            },
            status => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", status
            ))),
        }
    }

    pub async fn get_module_status(&self, name: &str) -> Result<ModuleStatus, ClientError> {
        let response = self.client
            .get(&format!("{}/modules/{}/status", self.base_url, name))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let status = response.json().await?;
                Ok(status)
            },
            StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            },
            status => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", status
            ))),
        }
    }

    pub async fn update_module_status(&self, name: &str, status: &ModuleStatus) -> Result<(), ClientError> {
        let response = self.client
            .put(&format!("{}/modules/{}/status", self.base_url, name))
            .json(&status)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::NOT_FOUND => {
                Err(ClientError::ModuleNotFound(name.to_string()))
            },
            status => Err(ClientError::InvalidResponse(format!(
                "Unexpected status code: {}", status
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_create_module() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/modules"))
            .respond_with(ResponseTemplate::new(StatusCode::CREATED.as_u16())
                .set_body_json(json!(1)))
            .mount(&mock_server)
            .await;

        let client = RegistrarClient::new(&mock_server.uri()).unwrap();

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

        let result = client.create_module(&module).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
