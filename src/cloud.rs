//! TerraTactics cloud agent stub (MQTT placeholder).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tracing::{debug, info};

use crate::buffer::OfflineBuffer;
use crate::config::CloudSection;

pub struct CloudAgent {
    enabled: bool,
    broker_url: Option<String>,
    client_id: Option<String>,
}

impl CloudAgent {
    pub fn from_config(cfg: &CloudSection) -> Self {
        Self {
            enabled: cfg.enabled,
            broker_url: cfg.broker_url.clone(),
            client_id: cfg.client_id.clone(),
        }
    }

    /// Periodically attempt to sync the offline buffer (no real MQTT yet).
    pub async fn run_sync_loop(&self, buffer: Arc<Mutex<OfflineBuffer>>) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if !self.enabled {
                debug!("cloud agent disabled; skipping sync");
                continue;
            }
            let mut buf = buffer.lock().expect("buffer mutex");
            match buf.drain_batch(32) {
                Ok(items) if items.is_empty() => {}
                Ok(items) => {
                    info!(
                        count = items.len(),
                        broker = ?self.broker_url,
                        client = ?self.client_id,
                        "cloud sync stub — would publish MQTT then delete"
                    );
                    // Skeleton: do not delete until a real broker ACK exists.
                    let _ = items;
                }
                Err(err) => debug!(error = %err, "buffer drain failed"),
            }
        }
    }
}
