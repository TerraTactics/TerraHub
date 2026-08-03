//! In-memory stub radio for bring-up without hardware.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::Notify;
use tracing::debug;

use super::{RadioError, RadioTransport, WireFrame};

/// Loopback-capable stub: `inject` queues frames for `recv`; `send` logs and optionally echoes.
pub struct StubRadio {
    inbound: Mutex<VecDeque<WireFrame>>,
    notify: Notify,
}

impl StubRadio {
    pub fn new() -> Self {
        Self {
            inbound: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    /// Test helper: push a frame as if the coprocessor received it over the air.
    pub fn inject(&self, frame: WireFrame) {
        self.inbound.lock().expect("stub queue").push_back(frame);
        self.notify.notify_one();
    }
}

impl Default for StubRadio {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RadioTransport for StubRadio {
    fn name(&self) -> &str {
        "stub"
    }

    async fn send(&self, frame: &[u8]) -> Result<(), RadioError> {
        debug!(len = frame.len(), "stub radio TX (dropped)");
        Ok(())
    }

    async fn recv(&self) -> Result<WireFrame, RadioError> {
        loop {
            if let Some(frame) = self.inbound.lock().expect("stub mutex").pop_front() {
                return Ok(frame);
            }
            self.notify.notified().await;
        }
    }
}
