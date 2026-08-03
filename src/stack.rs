//! TerraLink stack: decode RX frames, drive registry / buffer hooks, apply claims.

use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use terralink::{
    decode_frame, encode_frame, ConfigurationPayload, DiscoveryPayload, Frame, Header, PacketType,
    ROUTING_ADDR_BROADCAST, ROUTING_ADDR_UNSET,
};

use crate::buffer::OfflineBuffer;
use crate::radio::RadioTransport;
use crate::registry::{ClaimState, DeviceRecord, DeviceRegistry};

pub struct TerraLinkStack {
    radio: Arc<dyn RadioTransport>,
    registry: Arc<RwLock<DeviceRegistry>>,
    buffer: Arc<Mutex<OfflineBuffer>>,
    hub_routing_addr: u16,
    tx_sequence: AtomicU16,
}

impl TerraLinkStack {
    pub fn new(
        radio: Arc<dyn RadioTransport>,
        registry: Arc<RwLock<DeviceRegistry>>,
        buffer: Arc<Mutex<OfflineBuffer>>,
        hub_routing_addr: u16,
    ) -> Self {
        Self {
            radio,
            registry,
            buffer,
            hub_routing_addr,
            tx_sequence: AtomicU16::new(1),
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

    async fn handle_frame(&self, frame: Frame) {
        debug!(
            packet_type = ?frame.header.packet_type,
            src = frame.header.src_addr,
            seq = frame.header.sequence,
            "RX frame"
        );

        match frame.header.packet_type {
            PacketType::Discovery => {
                let (identity, node_hint) = match DiscoveryPayload::decode(frame.payload.as_slice())
                {
                    Ok(p) => (p.identity, Some(p.node_class.as_u8())),
                    Err(_) => (
                        parse_discovery_identity(frame.payload.as_slice()).unwrap_or_else(|| {
                            format!("unknown-{:04X}", frame.header.src_addr)
                        }),
                        None,
                    ),
                };
                let mut reg = self.registry.write().await;
                let existing = reg.get(&identity).cloned();
                let claim = existing
                    .as_ref()
                    .map(|e| e.claim)
                    .unwrap_or(ClaimState::Pending);
                let routing_addr = if claim == ClaimState::Claimed {
                    existing
                        .as_ref()
                        .map(|e| e.routing_addr)
                        .unwrap_or(frame.header.src_addr)
                } else {
                    frame.header.src_addr
                };
                let node_class = node_hint.or_else(|| existing.as_ref().and_then(|e| e.node_class));
                reg.upsert(DeviceRecord {
                    identity: identity.clone(),
                    routing_addr,
                    claim,
                    last_seen_seq: frame.header.sequence,
                    node_class,
                });
                info!(%identity, "discovery noted (claim via TerraTactics cloud or admin stub)");
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
                debug!(?other, "packet type not handled yet");
            }
        }
    }

    /// Apply a cloud/admin claim: mark registry claimed and send Configuration `0x07`.
    pub async fn apply_claim(&self, identity: &str, routing_addr: u16) -> Result<()> {
        if routing_addr == ROUTING_ADDR_UNSET || routing_addr == ROUTING_ADDR_BROADCAST {
            return Err(anyhow!(
                "routing_addr must be in 0x0001..=0xFFFE (got 0x{routing_addr:04X})"
            ));
        }

        let dst_addr = {
            let mut reg = self.registry.write().await;
            let Some(existing) = reg.get(identity).cloned() else {
                return Err(anyhow!("unknown device identity: {identity}"));
            };
            let dst = if existing.routing_addr != ROUTING_ADDR_UNSET {
                existing.routing_addr
            } else {
                ROUTING_ADDR_BROADCAST
            };
            if !reg.mark_claimed(identity, routing_addr) {
                return Err(anyhow!("failed to mark claimed: {identity}"));
            }
            dst
        };

        let payload = ConfigurationPayload::new(routing_addr, identity)
            .encode()
            .map_err(|e| anyhow!("encode configuration payload: {e:?}"))?;

        let seq = self.tx_sequence.fetch_add(1, Ordering::Relaxed);
        let header = Header::new(
            PacketType::Configuration,
            self.hub_routing_addr,
            dst_addr,
            seq,
            8,
        )
        .with_ack_req(true);
        let frame = Frame::new(header, &payload)
            .map_err(|e| anyhow!("build configuration frame: {e:?}"))?;

        let mut buf = vec![0u8; 12 + payload.len() + 2];
        let n = encode_frame(&frame, &mut buf)
            .map_err(|e| anyhow!("encode configuration frame: {e:?}"))?;
        self.radio
            .send(&buf[..n])
            .await
            .map_err(|e| anyhow!("radio send configuration: {e}"))?;

        info!(%identity, routing_addr, dst = dst_addr, "claim applied — Configuration 0x07 sent");
        Ok(())
    }
}

/// Minimal parse of Discovery MVP payload: `identity_len` + UTF-8 identity.
fn parse_discovery_identity(payload: &[u8]) -> Option<String> {
    let len = *payload.first()? as usize;
    let bytes = payload.get(1..1 + len)?;
    String::from_utf8(bytes.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radio::stub::StubRadio;
    use tempfile::tempdir;
    use terralink::{DiscoveryPayload, NodeClass};

    #[tokio::test]
    async fn discovery_then_claim_sends_configuration() {
        let radio = Arc::new(StubRadio::new());
        let registry = Arc::new(RwLock::new(DeviceRegistry::new()));
        let dir = tempdir().unwrap();
        let buffer = Arc::new(Mutex::new(
            OfflineBuffer::open(&dir.path().join("t.db")).unwrap(),
        ));
        let stack = TerraLinkStack::new(
            Arc::clone(&radio) as Arc<dyn RadioTransport>,
            Arc::clone(&registry),
            buffer,
            1,
        );

        let disc = DiscoveryPayload::new("TL-000127", NodeClass::Soil, 1, 0);
        let payload = disc.encode().unwrap();
        let header = Header::new(
            PacketType::Discovery,
            ROUTING_ADDR_UNSET,
            ROUTING_ADDR_BROADCAST,
            7,
            8,
        );
        let frame = Frame::new(header, &payload).unwrap();
        let mut raw = vec![0u8; 64];
        let n = encode_frame(&frame, &mut raw).unwrap();
        radio.inject(raw[..n].to_vec());

        // Drive one RX iteration via handle_frame path used by the loop.
        let decoded = decode_frame(&raw[..n]).unwrap();
        stack.handle_frame(decoded).await;

        {
            let reg = registry.read().await;
            let rec = reg.get("TL-000127").unwrap();
            assert_eq!(rec.claim, ClaimState::Pending);
        }

        stack.apply_claim("TL-000127", 0x0042).await.unwrap();
        {
            let reg = registry.read().await;
            let rec = reg.get("TL-000127").unwrap();
            assert_eq!(rec.claim, ClaimState::Claimed);
            assert_eq!(rec.routing_addr, 0x0042);
        }

        let tx = radio.drain_tx();
        assert_eq!(tx.len(), 1);
        let sent = decode_frame(&tx[0]).unwrap();
        assert_eq!(sent.header.packet_type, PacketType::Configuration);
        let cfg = ConfigurationPayload::decode(sent.payload.as_slice()).unwrap();
        assert_eq!(cfg.routing_addr, 0x0042);
        assert_eq!(cfg.identity, "TL-000127");
    }
}
