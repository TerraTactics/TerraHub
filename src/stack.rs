//! TerraLink stack: decode RX frames, drive registry / buffer hooks.

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use terralink::{decode_frame, PacketType};

use crate::buffer::OfflineBuffer;
use crate::radio::RadioTransport;
use crate::registry::{ClaimState, DeviceRecord, DeviceRegistry};

pub struct TerraLinkStack {
    radio: Arc<dyn RadioTransport>,
    registry: Arc<RwLock<DeviceRegistry>>,
    buffer: Arc<Mutex<OfflineBuffer>>,
}

impl TerraLinkStack {
    pub fn new(
        radio: Arc<dyn RadioTransport>,
        registry: Arc<RwLock<DeviceRegistry>>,
        buffer: Arc<Mutex<OfflineBuffer>>,
    ) -> Self {
        Self {
            radio,
            registry,
            buffer,
        }
    }

    /// Forever loop: receive wire frames and dispatch by packet type.
    pub async fn run_rx_loop(self: Arc<Self>) -> Result<()> {
        info!(backend = self.radio.name(), "TerraLink RX loop started");
        loop {
            let raw = self.radio.recv().await.context("radio recv")?;
            match decode_frame(&raw) {
                Ok(frame) => self.handle_frame(frame).await,
                Err(err) => warn!(?err, len = raw.len(), "dropping undecodable frame"),
            }
        }
    }

    async fn handle_frame(&self, frame: terralink::Frame) {
        debug!(
            packet_type = ?frame.header.packet_type,
            src = frame.header.src_addr,
            seq = frame.header.sequence,
            "RX frame"
        );

        match frame.header.packet_type {
            PacketType::Discovery => {
                let identity = parse_discovery_identity(frame.payload.as_slice())
                    .unwrap_or_else(|| format!("unknown-{:04X}", frame.header.src_addr));
                let mut reg = self.registry.write().await;
                reg.upsert(DeviceRecord {
                    identity: identity.clone(),
                    routing_addr: frame.header.src_addr,
                    claim: ClaimState::Pending,
                    last_seen_seq: frame.header.sequence,
                });
                info!(%identity, "discovery noted (claim via FarmPilot)");
            }
            PacketType::SensorData | PacketType::Alarm => {
                if let Err(err) = self.buffer.lock().expect("buffer mutex").enqueue_telemetry(
                    frame.header.src_addr,
                    frame.header.packet_type.as_u8(),
                    frame.payload.as_slice(),
                ) {
                    warn!(error = %err, "buffer enqueue failed");
                }
            }
            PacketType::Acknowledgement => {
                debug!(seq = frame.header.sequence, "ACK received");
            }
            other => {
                debug!(?other, "packet type not handled in skeleton");
            }
        }
    }
}

/// Minimal parse of Discovery MVP payload: `identity_len` + UTF-8 identity.
fn parse_discovery_identity(payload: &[u8]) -> Option<String> {
    let len = *payload.first()? as usize;
    let bytes = payload.get(1..1 + len)?;
    String::from_utf8(bytes.to_vec()).ok()
}
