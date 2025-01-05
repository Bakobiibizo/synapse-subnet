use std::net::SocketAddr;
use std::path::PathBuf;
use synapse_registrar::LocalRegistry;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the registry
    let storage_path = std::env::var("STORAGE_PATH")
        .unwrap_or_else(|_| "/tmp/synapse-registry".to_string());
    let registry = LocalRegistry::new(PathBuf::from(storage_path));

    // Create the router
    let app = synapse_registrar_api::create_router(registry);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Starting server on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
