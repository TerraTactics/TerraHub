//! Radio transport abstraction (UART / USB-serial coprocessor).

use async_trait::async_trait;
use thiserror::Error;

pub mod stub;
pub mod uart;

/// Errors from the radio backend.
#[derive(Debug, Error)]
pub enum RadioError {
    #[error("radio not connected")]
    NotConnected,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

/// Frame bytes as exchanged with the LoRa coprocessor (TerraLink on-wire frames).
pub type WireFrame = Vec<u8>;

/// Abstract radio transport. Real backends talk UART/USB-serial to an ESP32+LoRa (or similar).
#[async_trait]
pub trait RadioTransport: Send + Sync {
    /// Human-readable backend name.
    fn name(&self) -> &str;

    /// Send a complete TerraLink frame to the air (via coprocessor).
    async fn send(&self, frame: &[u8]) -> Result<(), RadioError>;

    /// Receive the next frame (waits until one is available or an error occurs).
    async fn recv(&self) -> Result<WireFrame, RadioError>;
}
