//! Length-prefixed UART framing for TerraLink wire frames.
//!
//! On the serial link to the LoRa coprocessor, each TerraLink frame is wrapped as:
//!
//! ```text
//! | len_lo | len_hi | terra_link_frame_bytes... |
//! ```
//!
//! `len` is a little-endian `u16` equal to the TerraLink frame length (header + payload + CRC),
//! and must not exceed [`MAX_UART_FRAME`] (coprocessor MVP limit).

use super::RadioError;

/// Maximum TerraLink frame size accepted on the UART (header 12 + payload 200 + CRC 2).
pub const MAX_UART_FRAME: usize = 12 + 200 + 2;

/// Encode a TerraLink wire frame into a length-prefixed UART blob.
pub fn encode_uart_datagram(terralink_frame: &[u8]) -> Result<Vec<u8>, RadioError> {
    if terralink_frame.is_empty() {
        return Err(RadioError::Io("empty TerraLink frame".into()));
    }
    if terralink_frame.len() > MAX_UART_FRAME {
        return Err(RadioError::Io(format!(
            "frame too large for UART: {} > {}",
            terralink_frame.len(),
            MAX_UART_FRAME
        )));
    }
    let len = terralink_frame.len() as u16;
    let mut out = Vec::with_capacity(2 + terralink_frame.len());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(terralink_frame);
    Ok(out)
}

/// Streaming decoder for length-prefixed UART datagrams.
#[derive(Debug, Default)]
pub struct UartFrameDecoder {
    buf: Vec<u8>,
}

impl UartFrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push serial bytes; returns zero or more complete TerraLink frames.
    pub fn push(&mut self, data: &[u8]) -> Result<Vec<Vec<u8>>, RadioError> {
        self.buf.extend_from_slice(data);
        let mut frames = Vec::new();
        loop {
            if self.buf.len() < 2 {
                break;
            }
            let len = u16::from_le_bytes([self.buf[0], self.buf[1]]) as usize;
            if len == 0 || len > MAX_UART_FRAME {
                // Resync: drop one byte and keep scanning.
                self.buf.remove(0);
                continue;
            }
            if self.buf.len() < 2 + len {
                break;
            }
            frames.push(self.buf[2..2 + len].to_vec());
            self.buf.drain(..2 + len);
        }
        Ok(frames)
    }

    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let frame = vec![0x01, 0x02, 0x03, 0x04];
        let datagram = encode_uart_datagram(&frame).unwrap();
        assert_eq!(&datagram[0..2], &4u16.to_le_bytes());
        let mut dec = UartFrameDecoder::new();
        let out = dec.push(&datagram).unwrap();
        assert_eq!(out, vec![frame]);
    }

    #[test]
    fn split_across_reads() {
        let frame = b"abcdefgh".to_vec();
        let datagram = encode_uart_datagram(&frame).unwrap();
        let mut dec = UartFrameDecoder::new();
        assert!(dec.push(&datagram[..3]).unwrap().is_empty());
        let out = dec.push(&datagram[3..]).unwrap();
        assert_eq!(out, vec![frame]);
    }

    #[test]
    fn two_frames_one_chunk() {
        let a = encode_uart_datagram(b"aa").unwrap();
        let b = encode_uart_datagram(b"bbbb").unwrap();
        let mut chunk = a;
        chunk.extend_from_slice(&b);
        let mut dec = UartFrameDecoder::new();
        let out = dec.push(&chunk).unwrap();
        assert_eq!(out, vec![b"aa".to_vec(), b"bbbb".to_vec()]);
    }
}
