//! UART / USB-serial radio coprocessor backend (stub).

use async_trait::async_trait;
use tracing::warn;

use super::{RadioError, RadioTransport, WireFrame};

/// Placeholder for a future serial-attached LoRa coprocessor.
pub struct UartRadio {
    device: String,
    baud: u32,
}

impl UartRadio {
    pub fn new(device: impl Into<String>, baud: u32) -> Self {
        Self {
            device: device.into(),
            baud,
        }
    }
}

#[async_trait]
impl RadioTransport for UartRadio {
    fn name(&self) -> &str {
        "uart"
    }

    async fn send(&self, _frame: &[u8]) -> Result<(), RadioError> {
        warn!(device = %self.device, baud = self.baud, "UART radio send not implemented");
        Err(RadioError::NotImplemented(
            "UART radio: open serial and speak coprocessor framing",
        ))
    }

    async fn recv(&self) -> Result<WireFrame, RadioError> {
        Err(RadioError::NotImplemented(
            "UART radio: open serial and speak coprocessor framing",
        ))
    }
}
