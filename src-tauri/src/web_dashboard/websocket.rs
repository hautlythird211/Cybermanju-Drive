// WebSocket event bus for live push notifications.
// Uses a minimal RFC 6455 handshake implementation.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};

/// WebSocket magic constant for handshake.
const WS_MAGIC: &str = "258EAFA5-E914-47DA-95CA-5AB9D111CF85";

/// WebSocket opcodes.
const OP_TEXT: u8 = 0x01;
const OP_CLOSE: u8 = 0x08;
const OP_PING: u8 = 0x09;
const OP_PONG: u8 = 0x0A;

/// Check if an HTTP request is a WebSocket upgrade.
pub fn is_websocket_upgrade(headers: &HashMap<String, String>) -> bool {
    headers
        .get("upgrade")
        .map(|h| h.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

/// Perform WebSocket handshake on an upgraded TCP stream.
pub fn websocket_handshake(stream: &mut TcpStream, key: &str) -> std::io::Result<()> {
    let mut accept_input = Vec::with_capacity(key.len() + WS_MAGIC.len());
    accept_input.extend_from_slice(key.as_bytes());
    accept_input.extend_from_slice(WS_MAGIC.as_bytes());

    let mut hasher = Sha256::new();
    hasher.update(&accept_input);
    let accept_hash = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &hasher.finalize(),
    );

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_hash
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

/// Send a WebSocket text frame.
pub fn ws_send_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    let payload = text.as_bytes();
    let len = payload.len();

    let mut frame = Vec::with_capacity(2 + 8 + len);
    frame.push(0x80 | OP_TEXT); // FIN + TEXT

    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    frame.extend_from_slice(payload);
    stream.write_all(&frame)?;
    stream.flush()?;
    Ok(())
}

/// Read a WebSocket frame from the stream.
pub fn ws_read_frame(stream: &mut TcpStream) -> std::io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;

    let opcode = header[0] & 0x0F;
    let masked = header[1] & 0x80 != 0;
    let mut len = (header[1] & 0x7F) as u64;

    if len == 126 {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        len = u16::from_be_bytes(buf) as u64;
    } else if len == 127 {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        len = u64::from_be_bytes(buf);
    }

    let mask = if masked {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        Some(buf)
    } else {
        None
    };

    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload)?;

    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    match opcode {
        OP_TEXT => Ok(Some(payload)),
        OP_CLOSE => {
            let _ = stream.write_all(&[0x80 | OP_CLOSE, 0x00]);
            Ok(None)
        }
        OP_PING => {
            // Build pong response with proper length encoding
            let mut pong = vec![0x80 | OP_PONG];
            let len = payload.len();
            if len < 126 {
                pong.push(len as u8);
            } else if len < 65536 {
                pong.push(126);
                pong.extend_from_slice(&(len as u16).to_be_bytes());
            } else {
                pong.push(127);
                pong.extend_from_slice(&(len as u64).to_be_bytes());
            }
            pong.extend_from_slice(&payload);
            stream.write_all(&pong)?;
            Ok(Some(vec![]))
        }
        OP_PONG => {
            // Pong frame — acknowledgment, no action needed
            Ok(Some(vec![]))
        }
        _ => {
            // Unknown opcode — reject per RFC 6455 by closing the connection
            let _ = stream.write_all(&[0x80 | OP_CLOSE, 0x00]);
            Ok(None)
        }
    }
}

/// Shared event bus for broadcasting events to WebSocket clients.
pub struct WebSocketEventBus {
    subscribers: Mutex<Vec<TcpStream>>,
}

impl WebSocketEventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }

    /// Add a new WebSocket client subscriber.
    pub fn subscribe(&self, stream: TcpStream) {
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(stream);
        }
    }

    /// Broadcast a JSON event to all connected clients.
    pub fn broadcast(&self, json: &str) {
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.retain_mut(|stream| {
                ws_send_text(stream, json).is_ok()
            });
        }
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.subscribers
            .lock()
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

impl Default for WebSocketEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_websocket_upgrade() {
        let mut headers = HashMap::new();
        headers.insert("upgrade".to_string(), "websocket".to_string());
        headers.insert("connection".to_string(), "Upgrade".to_string());
        assert!(is_websocket_upgrade(&headers));

        headers.insert("upgrade".to_string(), "HTTP/2".to_string());
        assert!(!is_websocket_upgrade(&headers));
    }
}
