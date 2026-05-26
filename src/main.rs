use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

mod client;
mod error;
mod params;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr — stdout is reserved for MCP JSON-RPC messages.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    // In debug builds, watch our own binary for changes and exit when bacon
    // rebuilds it. Claude Code auto-restarts crashed MCP servers, so this
    // gives zero-restart hot-reload during development.
    #[cfg(debug_assertions)]
    if let Ok(exe) = std::env::current_exe()
        && let Ok(meta) = std::fs::metadata(&exe)
        && let Ok(initial_mtime) = meta.modified()
    {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if let Ok(m) = std::fs::metadata(&exe)
                    && m.modified().ok() != Some(initial_mtime)
                {
                    tracing::info!("binary changed — exiting for hot-reload");
                    std::process::exit(0);
                }
            }
        });
    }

    let server = server::MikrotikServer::from_env()?;
    tracing::info!("mikrotik-mcp starting");

    let svc = server.serve(stdio()).await?;
    svc.waiting().await?;

    Ok(())
}
