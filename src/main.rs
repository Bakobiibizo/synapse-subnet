mod interface;

use clap::Parser;
use interface::{ApiServer, Cli, GuiServer, core::db::Database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize database
    let db = Database::connect("sqlite:interface.db").await?;
    
    // Run migrations
    db.run_migrations().await?;

    // Start API server in background
    let api_server = ApiServer::new(db.clone()).await?;
    tokio::spawn(async move {
        if let Err(e) = api_server.run("0.0.0.0:3000").await {
            eprintln!("API server error: {}", e);
        }
    });

    // Start GUI server in background
    let gui_server = GuiServer::new(db.clone()).await?;
    tokio::spawn(async move {
        if let Err(e) = gui_server.run("0.0.0.0:8080").await {
            eprintln!("GUI server error: {}", e);
        }
    });

    // Run CLI
    cli.run().await?;

    Ok(())
}
