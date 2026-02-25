mod aseprite;
mod lua_helpers;
mod server;
mod tools;
mod utils;

use anyhow::Result;
use rmcp::ServiceExt;
use server::AsepriteServer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is reserved for MCP JSON-RPC protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    info!(
        "Starting Aseprite MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create the server (this also locates the Aseprite executable)
    let server = AsepriteServer::new()?;

    // Start MCP transport over stdio
    let transport = rmcp::transport::io::stdio();
    let service = server.serve(transport).await?;

    info!("Aseprite MCP Server is running. Waiting for requests...");

    // Wait until the service is shut down
    service.waiting().await?;

    info!("Aseprite MCP Server shut down.");
    Ok(())
}
