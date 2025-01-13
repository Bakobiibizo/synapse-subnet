use std::path::PathBuf;
use std::sync::Arc;
use axum::{
    extract::{Path as AxumPath, State},
    response::{Response},
    routing::get,
    Router,
    Json,
    http::StatusCode,
};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use serde::Serialize;
use tokio::fs;
use tokio::process::Command;
use tempfile::TempDir;
use tower_http::services::ServeDir;
use anyhow::Result;
use crate::registry::Registry;
use registrar_core::Module;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

#[derive(Serialize)]
struct InstallationPackage {
    package: String,  // base64 encoded
    hash: String,
    metadata: ModuleMetadata,
}

#[derive(Serialize)]
struct ModuleMetadata {
    name: String,
    version: String,
    repo_url: String,
    branch: String,
}

pub fn router(registry: Arc<Registry>) -> Router {
    Router::new()
        .route("/subnet-modules/:name/config", get(get_config))
        .route("/subnet-modules/:name/env-template", get(get_env_template))
        .route("/subnet-modules/:name/package", get(get_installation_package))
        .nest_service("/v2", ServeDir::new(CONFIG_DIR))
        .with_state(registry)
}

#[axum::debug_handler]
async fn get_config(
    AxumPath(name): AxumPath<String>,
    _state: State<Arc<Registry>>,
) -> Result<Response<String>, StatusCode> {
    let config_path = PathBuf::from(CONFIG_DIR)
        .join(&name)
        .join("config.yaml");
    
    match fs::read_to_string(config_path).await {
        Ok(content) => Ok(Response::builder()
            .header("content-type", "application/yaml")
            .body(content)
            .unwrap()),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[axum::debug_handler]
async fn get_env_template(
    AxumPath(name): AxumPath<String>,
    _state: State<Arc<Registry>>,
) -> Result<Response<String>, StatusCode> {
    let env_path = PathBuf::from(CONFIG_DIR)
        .join(&name)
        .join(".env.example");
    
    match fs::read_to_string(env_path).await {
        Ok(content) => Ok(Response::builder()
            .header("content-type", "text/plain")
            .body(content)
            .unwrap()),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[axum::debug_handler]
async fn get_installation_package(
    AxumPath(name): AxumPath<String>,
    State(registry): State<Arc<Registry>>,
) -> Result<Json<InstallationPackage>, StatusCode> {
    let registry_module = registry.get_module(&name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let module: Module = registry_module.clone().into();
    let package_dir = PathBuf::from(CONFIG_DIR).join(&name);
    
    // Copy files to temp dir
    let temp_dir = TempDir::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let package_dir = temp_dir.path().join(&name);
    fs::create_dir_all(&package_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    fs::copy(
        PathBuf::from(CONFIG_DIR).join(&name).join("config.yaml"),
        package_dir.join("config.yaml"),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    fs::copy(
        PathBuf::from(CONFIG_DIR).join(&name).join(".env.example"),
        package_dir.join(".env.example"),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate installer script
    let installer = generate_installer_script(&module)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write(package_dir.join("install.sh"), installer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create tar.gz
    let output = Command::new("tar")
        .arg("-czf")
        .arg("package.tar.gz")
        .arg("-C")
        .arg(temp_dir.path())
        .arg(&name)
        .output()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let package_bytes = fs::read(package_dir.join("package.tar.gz"))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let package = general_purpose::STANDARD.encode(&package_bytes);
    let mut hasher = Sha256::new();
    hasher.update(&package_bytes);
    let hash = general_purpose::STANDARD.encode(hasher.finalize());

    let metadata = ModuleMetadata {
        name: registry_module.name.clone(),
        version: registry_module.version.clone(),
        repo_url: registry_module.repo_url.clone(),
        branch: registry_module.branch.clone(),
    };

    Ok(Json(InstallationPackage {
        package,
        hash,
        metadata,
    }))
}

fn generate_installer_script(module: &Module) -> Result<String> {
    Ok(format!(r#"#!/bin/bash
set -e

echo "Installing {} module..."
mkdir -p ~/.synapsis/modules/{}
tar xzf package.tar.gz -C ~/.synapsis/modules/{}

echo "Done! Module installed at ~/.synapsis/modules/{}"
"#, module.name, module.name, module.name, module.name))
}
