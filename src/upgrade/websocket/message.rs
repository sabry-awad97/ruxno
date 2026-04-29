//! WebSocket message types

use bytes::Bytes;

/// WebSocket message
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Bytes),
    /// Ping
    Ping(Vec<u8>),
    /// Pong
    Pong(Vec<u8>),
    /// Close
    Close,
}

impl Message {
    /// Create text message
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Create binary message
    pub fn binary(data: impl Into<Bytes>) -> Self {
        Self::Binary(data.into())
    }

    /// Check if message is text
    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    /// Check if message is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }
}
