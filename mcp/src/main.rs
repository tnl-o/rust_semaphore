mod client;
mod protocol;
mod server;
mod tools;

use anyhow::Result;
use client::VelumClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (e.g. during local development)
    let _ = dotenvy::dotenv();

    // Initialise tracing — write to stderr so stdout stays clean for MCP stdio
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "velum_mcp=info".into())
                .as_str(),
        )
        .init();

    let client = VelumClient::from_env()?;

    // Transport selection: MCP_TRANSPORT=http or MCP_TRANSPORT=stdio (default)
    let transport = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".into());
    let port: u16 = std::env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8500);

    match transport.as_str() {
        "http" => server::run_http(client, port).await,
        _ => server::run_stdio(client).await,
    }
}
