//! TerraHub daemon entrypoint.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use terrahub::admin;
use terrahub::buffer::OfflineBuffer;
use terrahub::cloud::CloudAgent;
use terrahub::config::HubConfig;
use terrahub::radio::{stub::StubRadio, RadioTransport};
use terrahub::registry::DeviceRegistry;
use terrahub::stack::TerraLinkStack;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("terrahub=info".parse()?))
        .init();

    let config_path = std::env::args()
        .position(|a| a == "--config")
        .and_then(|i| std::env::args().nth(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/terrahub.example.toml"));

    let config = HubConfig::load(&config_path)
        .with_context(|| format!("load config from {}", config_path.display()))?;
    info!(identity = %config.hub.identity, "starting TerraHub");

    let radio: Arc<dyn RadioTransport> = Arc::new(StubRadio::new());
    let registry = Arc::new(RwLock::new(DeviceRegistry::new()));
    let buffer = OfflineBuffer::open(&config.buffer.sqlite_path)
        .with_context(|| format!("open sqlite at {}", config.buffer.sqlite_path.display()))?;
    // rusqlite::Connection is Send but not Sync — std::Mutex is the right wrapper.
    let buffer = Arc::new(Mutex::new(buffer));
    let cloud = Arc::new(CloudAgent::from_config(&config.cloud));
    let stack = Arc::new(TerraLinkStack::new(
        Arc::clone(&radio),
        Arc::clone(&registry),
        Arc::clone(&buffer),
    ));

    let stack_rx = Arc::clone(&stack);
    tokio::spawn(async move {
        if let Err(err) = stack_rx.run_rx_loop().await {
            warn!(error = %err, "TerraLink RX loop ended");
        }
    });

    let cloud_task = Arc::clone(&cloud);
    let buffer_task = Arc::clone(&buffer);
    tokio::spawn(async move {
        cloud_task.run_sync_loop(buffer_task).await;
    });

    info!(bind = %config.admin.bind, "admin HTTP listening (setup wizard stubs)");
    admin::serve(&config.admin.bind, Arc::clone(&registry), config.hub.identity.clone()).await?;

    Ok(())
}
