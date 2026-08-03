//! In-memory stub radio for bring-up without hardware.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::Notify;
use tracing::debug;

use super::{RadioError, RadioTransport, WireFrame};

/// Loopback-capable stub: `inject` queues frames for `recv`; `send` records TX for tests.
pub struct StubRadio {
    inbound: Mutex<VecDeque<WireFrame>>,
    outbound: Mutex<VecDeque<WireFrame>>,
    notify: Notify,
}

impl StubRadio {
    pub fn new() -> Self {
        Self {
            inbound: Mutex::new(VecDeque::new()),
            outbound: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    /// Test helper: push a frame as if the coprocessor received it over the air.
    pub fn inject(&self, frame: WireFrame) {
        self.inbound.lock().expect("stub mutex").push_back(frame);
        self.notify.notify_one();
    }

    /// Frames previously passed to [`RadioTransport::send`] (oldest first).
    pub fn drain_tx(&self) -> Vec<WireFrame> {
        self.outbound.lock().expect("stub outbound").drain(..).collect()
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
        debug!(len = frame.len(), "stub radio TX");
        self.outbound
            .lock()
            .expect("stub outbound")
            .push_back(frame.to_vec());
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
