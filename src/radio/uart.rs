//! UART / USB-serial radio coprocessor backend.
//!
//! Speaks length-prefixed TerraLink frames (see [`super::framing`]) over a serial port
//! such as `/dev/ttyUSB0`, `/dev/ttyAMA0`, or Windows `COM3`.
//!
//! For bring-up without hardware, use [`UartRadio::loopback`] or [`UartRadio::loopback_pair`] —
//! the same framing path over an in-memory channel.

use std::collections::VecDeque;
use std::sync::Mutex as StdMutex;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex, Notify};
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, info};

use super::framing::{encode_uart_datagram, UartFrameDecoder};
use super::{RadioError, RadioTransport, WireFrame};

enum Backend {
    /// Real USB-serial / UART device.
    Serial {
        device: String,
        baud: u32,
        port: Mutex<tokio_serial::SerialStream>,
    },
    /// In-memory duplex using the same UART framing (tests / no hardware).
    Loopback {
        tx: StdMutex<mpsc::UnboundedSender<Vec<u8>>>,
        rx: Mutex<mpsc::UnboundedReceiver<Vec<u8>>>,
    },
}

/// UART / USB-serial radio transport.
pub struct UartRadio {
    backend: Backend,
    decoder: StdMutex<UartFrameDecoder>,
    inbound: StdMutex<VecDeque<WireFrame>>,
    inbound_notify: Notify,
}

impl UartRadio {
    /// Open a serial device (e.g. `/dev/ttyUSB0` or `COM3`) at `baud`.
    pub fn open(device: impl Into<String>, baud: u32) -> Result<Self, RadioError> {
        let device = device.into();
        let port = tokio_serial::new(&device, baud)
            .open_native_async()
            .map_err(|e| RadioError::Io(format!("open serial {device} @ {baud}: {e}")))?;
        info!(%device, baud, "UART radio opened");
        Ok(Self {
            backend: Backend::Serial {
                device,
                baud,
                port: Mutex::new(port),
            },
            decoder: StdMutex::new(UartFrameDecoder::new()),
            inbound: StdMutex::new(VecDeque::new()),
            inbound_notify: Notify::new(),
        })
    }

    /// Single-ended in-memory radio; feed RX via [`Self::inject_uart_bytes`].
    pub fn loopback() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            backend: Backend::Loopback {
                tx: StdMutex::new(tx),
                rx: Mutex::new(rx),
            },
            decoder: StdMutex::new(UartFrameDecoder::new()),
            inbound: StdMutex::new(VecDeque::new()),
            inbound_notify: Notify::new(),
        }
    }

    /// Two ends of a framed serial pipe: A’s TX becomes B’s RX and vice versa.
    pub fn loopback_pair() -> (Self, Self) {
        let (a_to_b_tx, a_to_b_rx) = mpsc::unbounded_channel();
        let (b_to_a_tx, b_to_a_rx) = mpsc::unbounded_channel();
        let a = Self {
            backend: Backend::Loopback {
                tx: StdMutex::new(a_to_b_tx),
                rx: Mutex::new(b_to_a_rx),
            },
            decoder: StdMutex::new(UartFrameDecoder::new()),
            inbound: StdMutex::new(VecDeque::new()),
            inbound_notify: Notify::new(),
        };
        let b = Self {
            backend: Backend::Loopback {
                tx: StdMutex::new(b_to_a_tx),
                rx: Mutex::new(a_to_b_rx),
            },
            decoder: StdMutex::new(UartFrameDecoder::new()),
            inbound: StdMutex::new(VecDeque::new()),
            inbound_notify: Notify::new(),
        };
        (a, b)
    }

    /// Inject raw UART bytes (as if read from the serial port) — primarily for tests.
    pub fn inject_uart_bytes(&self, data: &[u8]) -> Result<(), RadioError> {
        let frames = self.decoder.lock().expect("decoder").push(data)?;
        self.push_decoded(frames);
        Ok(())
    }

    fn push_decoded(&self, frames: Vec<WireFrame>) {
        if frames.is_empty() {
            return;
        }
        {
            let mut q = self.inbound.lock().expect("inbound");
            for f in frames {
                q.push_back(f);
            }
        }
        self.inbound_notify.notify_waiters();
    }

    async fn fill_inbound(&self) -> Result<(), RadioError> {
        match &self.backend {
            Backend::Serial { device, port, .. } => {
                let mut buf = [0u8; 512];
                let mut port = port.lock().await;
                let n = port.read(&mut buf).await.map_err(|e| {
                    RadioError::Io(format!("serial read on {device}: {e}"))
                })?;
                if n == 0 {
                    return Err(RadioError::Io(format!("serial EOF on {device}")));
                }
                drop(port);
                let frames = self.decoder.lock().expect("decoder").push(&buf[..n])?;
                self.push_decoded(frames);
                Ok(())
            }
            Backend::Loopback { rx, .. } => {
                let chunk = {
                    let mut rx = rx.lock().await;
                    rx.recv()
                        .await
                        .ok_or_else(|| RadioError::Io("loopback disconnected".into()))?
                };
                let frames = self.decoder.lock().expect("decoder").push(&chunk)?;
                self.push_decoded(frames);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl RadioTransport for UartRadio {
    fn name(&self) -> &str {
        match &self.backend {
            Backend::Serial { .. } => "uart",
            Backend::Loopback { .. } => "uart-loopback",
        }
    }

    async fn send(&self, frame: &[u8]) -> Result<(), RadioError> {
        let datagram = encode_uart_datagram(frame)?;
        match &self.backend {
            Backend::Serial { device, baud, port } => {
                let mut port = port.lock().await;
                port.write_all(&datagram)
                    .await
                    .map_err(|e| RadioError::Io(format!("serial write {device}@{baud}: {e}")))?;
                port.flush()
                    .await
                    .map_err(|e| RadioError::Io(format!("serial flush {device}: {e}")))?;
                debug!(len = frame.len(), %device, "UART TX");
                Ok(())
            }
            Backend::Loopback { tx, .. } => {
                tx.lock()
                    .expect("tx")
                    .send(datagram)
                    .map_err(|_| RadioError::Io("loopback peer gone".into()))?;
                debug!(len = frame.len(), "UART loopback TX");
                Ok(())
            }
        }
    }

    async fn recv(&self) -> Result<WireFrame, RadioError> {
        loop {
            if let Some(frame) = self.inbound.lock().expect("inbound").pop_front() {
                return Ok(frame);
            }
            self.fill_inbound().await?;
        }
    }
}

/// Build radio from hub config (`stub`, `uart`, `usb-serial`, or `loopback`).
pub fn build_radio(
    backend: &str,
    device: Option<&str>,
    baud: Option<u32>,
) -> Result<std::sync::Arc<dyn RadioTransport>, RadioError> {
    match backend {
        "stub" => Ok(std::sync::Arc::new(super::stub::StubRadio::new())),
        "loopback" => Ok(std::sync::Arc::new(UartRadio::loopback())),
        "uart" | "usb-serial" => {
            let device = device.ok_or(RadioError::Io(
                "radio.device required for uart/usb-serial (e.g. /dev/ttyUSB0 or COM3)".into(),
            ))?;
            let baud = baud.unwrap_or(115_200);
            Ok(std::sync::Arc::new(UartRadio::open(device, baud)?))
        }
        other => Err(RadioError::Io(format!("unknown radio.backend: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loopback_pair_framed_exchange() {
        let (a, b) = UartRadio::loopback_pair();
        let payload = vec![0xAA, 0xBB, 0xCC, 0xDD];
        a.send(&payload).await.unwrap();
        let got = b.recv().await.unwrap();
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn inject_uart_bytes_decodes() {
        let radio = UartRadio::loopback();
        let frame = vec![1, 2, 3, 4, 5];
        let datagram = encode_uart_datagram(&frame).unwrap();
        radio.inject_uart_bytes(&datagram).unwrap();
        let got = radio.recv().await.unwrap();
        assert_eq!(got, frame);
    }
}
