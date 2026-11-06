use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

mod client;
mod config;
mod error;
mod params;
mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Log to stderr — stdout is reserved for MCP JSON-RPC messages.
    tracing_subscriber::fmt()
        .with_env_filter(env_filter(std::env::var("RUST_LOG").ok().as_deref()))
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

/// `info` unless `RUST_LOG` says otherwise: startup warnings and the one-line
/// record of each write must not depend on the user setting `RUST_LOG`.
fn env_filter(rust_log: Option<&str>) -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(rust_log.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_at_info_unless_rust_log_is_set() {
        assert_eq!(env_filter(None).max_level_hint(), Some(LevelFilter::INFO));
        assert_eq!(
            env_filter(Some("")).max_level_hint(),
            Some(LevelFilter::INFO)
        );
        assert_eq!(
            env_filter(Some("debug")).max_level_hint(),
            Some(LevelFilter::DEBUG)
        );
        assert_eq!(
            env_filter(Some("warn")).max_level_hint(),
            Some(LevelFilter::WARN)
        );
    }
}
