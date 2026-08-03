//! TerraTactics cloud agent stub (MQTT placeholder + claim directives).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tracing::{debug, info};

use crate::buffer::OfflineBuffer;
use crate::config::CloudSection;
use crate::stack::TerraLinkStack;

/// Claim instruction as would arrive from the TerraTactics cloud after farmer approval.
#[derive(Debug, Clone)]
pub struct ClaimDirective {
    pub identity: String,
    pub routing_addr: u16,
}

pub struct CloudAgent {
    enabled: bool,
    broker_url: Option<String>,
    client_id: Option<String>,
    /// Stub inbox for claim directives (cloud → hub).
    pending_claims: Mutex<Vec<ClaimDirective>>,
}

impl CloudAgent {
    pub fn from_config(cfg: &CloudSection) -> Self {
        Self {
            enabled: cfg.enabled,
            broker_url: cfg.broker_url.clone(),
            client_id: cfg.client_id.clone(),
            pending_claims: Mutex::new(Vec::new()),
        }
    }

    /// Queue a claim as if the TerraTactics cloud pushed it (admin/tests use this stub).
    pub fn enqueue_claim(&self, identity: impl Into<String>, routing_addr: u16) {
        let identity = identity.into();
        info!(%identity, routing_addr, "cloud stub: claim directive queued");
        self.pending_claims
            .lock()
            .expect("claims")
            .push(ClaimDirective {
                identity,
                routing_addr,
            });
    }

    /// Periodically attempt to sync the offline buffer and apply stub claims.
    pub async fn run_sync_loop(&self, buffer: Arc<Mutex<OfflineBuffer>>, stack: Arc<TerraLinkStack>) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let claims: Vec<ClaimDirective> = {
                let mut q = self.pending_claims.lock().expect("claims");
                q.drain(..).collect()
            };
            for claim in claims {
                if let Err(err) = stack
                    .apply_claim(&claim.identity, claim.routing_addr)
                    .await
                {
                    debug!(error = %err, identity = %claim.identity, "cloud claim apply failed");
                }
            }

            if !self.enabled {
                debug!("cloud agent disabled; skipping MQTT sync");
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
                    let _ = items;
                }
                Err(err) => debug!(error = %err, "buffer drain failed"),
            }
        }
    }
}
