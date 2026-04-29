//! WebSocket frame handling
//!
//! Implements WebSocket frame parsing and encoding according to RFC 6455.
//!
//! # Frame Format (RFC 6455 Section 5.2)
//!
//! ```text
//!  0                   1                   2                   3
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-------+-+-------------+-------------------------------+
//! |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
//! |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
//! |N|V|V|V|       |S|             |   (if payload len==126/127)   |
//! | |1|2|3|       |K|             |                               |
//! +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
//! |     Extended payload length continued, if payload len == 127  |
//! + - - - - - - - - - - - - - - - +-------------------------------+
//! |                               |Masking-key, if MASK set to 1  |
//! +-------------------------------+-------------------------------+
//! | Masking-key (continued)       |          Payload Data         |
//! +-------------------------------- - - - - - - - - - - - - - - - +
//! :                     Payload Data continued ...                :
//! + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
//! |                     Payload Data continued ...                |
//! +---------------------------------------------------------------+
//! ```
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::upgrade::websocket::{FrameHandler, Message};
//!
//! // Encode a message to a frame
//! let msg = Message::text("Hello");
//! let frame = FrameHandler::encode(&msg);
//!
//! // Decode a frame to a message
//! let decoded = FrameHandler::decode(&frame);
//! ```

use crate::upgrade::websocket::Message;

/// WebSocket frame opcodes (RFC 6455 Section 5.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    /// Continuation frame
    Continuation = 0x0,
    /// Text frame
    Text = 0x1,
    /// Binary frame
    Binary = 0x2,
    /// Connection close
    Close = 0x8,
    /// Ping
    Ping = 0x9,
    /// Pong
    Pong = 0xA,
}

impl Opcode {
    /// Create opcode from u8
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opcode = Opcode::from_u8(0x1);
    /// assert_eq!(opcode, Some(Opcode::Text));
    /// ```
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte & 0x0F {
            0x0 => Some(Opcode::Continuation),
            0x1 => Some(Opcode::Text),
            0x2 => Some(Opcode::Binary),
            0x8 => Some(Opcode::Close),
            0x9 => Some(Opcode::Ping),
            0xA => Some(Opcode::Pong),
            _ => None,
        }
    }

    /// Check if opcode is a control frame
    ///
    /// Control frames: Close, Ping, Pong (opcodes 0x8-0xF)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(Opcode::Ping.is_control());
    /// assert!(!Opcode::Text.is_control());
    /// ```
    pub fn is_control(self) -> bool {
        matches!(self, Opcode::Close | Opcode::Ping | Opcode::Pong)
    }
}

/// WebSocket frame
///
/// Represents a WebSocket frame with all its components according to RFC 6455.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::Frame;
///
/// let frame = Frame {
///     fin: true,
///     opcode: Opcode::Text,
///     mask: None,
///     payload: b"Hello".to_vec(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// FIN bit - indicates final fragment
    pub fin: bool,
    /// RSV1 bit - reserved for extensions
    pub rsv1: bool,
    /// RSV2 bit - reserved for extensions
    pub rsv2: bool,
    /// RSV3 bit - reserved for extensions
    pub rsv3: bool,
    /// Frame opcode
    pub opcode: Opcode,
    /// Masking key (4 bytes) - required for client-to-server frames
    pub mask: Option<[u8; 4]>,
    /// Payload data
    pub payload: Vec<u8>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: Opcode::Text,
            mask: None,
            payload: Vec::new(),
        }
    }
}

impl Frame {
    /// Create a new frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::new(Opcode::Text, b"Hello".to_vec());
    /// ```
    pub fn new(opcode: Opcode, payload: Vec<u8>) -> Self {
        Self {
            fin: true,
            opcode,
            payload,
            ..Default::default()
        }
    }

    /// Create a text frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::text("Hello");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(Opcode::Text, text.into().into_bytes())
    }

    /// Create a binary frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::binary(vec![1, 2, 3]);
    /// ```
    pub fn binary(data: Vec<u8>) -> Self {
        Self::new(Opcode::Binary, data)
    }

    /// Create a ping frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::ping(vec![]);
    /// ```
    pub fn ping(data: Vec<u8>) -> Self {
        Self::new(Opcode::Ping, data)
    }

    /// Create a pong frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::pong(vec![]);
    /// ```
    pub fn pong(data: Vec<u8>) -> Self {
        Self::new(Opcode::Pong, data)
    }

    /// Create a close frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::close();
    /// ```
    pub fn close() -> Self {
        Self::new(Opcode::Close, Vec::new())
    }

    /// Apply masking to payload
    ///
    /// XORs the payload with the masking key according to RFC 6455 Section 5.3.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut frame = Frame::text("Hello");
    /// frame.apply_mask([1, 2, 3, 4]);
    /// ```
    pub fn apply_mask(&mut self, mask: [u8; 4]) {
        for (i, byte) in self.payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
        self.mask = Some(mask);
    }

    /// Remove masking from payload
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut frame = Frame::text("Hello");
    /// frame.apply_mask([1, 2, 3, 4]);
    /// frame.unmask();
    /// ```
    pub fn unmask(&mut self) {
        if let Some(mask) = self.mask {
            for (i, byte) in self.payload.iter_mut().enumerate() {
                *byte ^= mask[i % 4];
            }
            self.mask = None;
        }
    }

    /// Check if frame is masked
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::text("Hello");
    /// assert!(!frame.is_masked());
    /// ```
    pub fn is_masked(&self) -> bool {
        self.mask.is_some()
    }

    /// Check if frame is a control frame
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::ping(vec![]);
    /// assert!(frame.is_control());
    /// ```
    pub fn is_control(&self) -> bool {
        self.opcode.is_control()
    }

    /// Get payload length
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame = Frame::text("Hello");
    /// assert_eq!(frame.payload_len(), 5);
    /// ```
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }
}

/// Frame handler
///
/// Provides methods for encoding and decoding WebSocket frames according to RFC 6455.
///
/// # Design
///
/// This is a stateless utility struct with static methods for frame operations.
/// The actual frame encoding/decoding will be implemented when hyper_tungstenite
/// integration is complete.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::{FrameHandler, Message};
///
/// // Encode a message
/// let msg = Message::text("Hello");
/// let frame_data = FrameHandler::encode(&msg);
///
/// // Decode a frame
/// let decoded = FrameHandler::decode(&frame_data);
/// ```
pub struct FrameHandler;

impl FrameHandler {
    /// Encode message to WebSocket frame bytes
    ///
    /// Converts a high-level Message into raw WebSocket frame bytes
    /// according to RFC 6455.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::text("Hello");
    /// let frame_bytes = FrameHandler::encode(&msg);
    /// ```
    ///
    /// # Note
    ///
    /// This is a stub implementation. Actual encoding will be handled by
    /// hyper_tungstenite when the server layer is complete.
    pub fn encode(_msg: &Message) -> Vec<u8> {
        // TODO: Implement actual frame encoding when hyper integration is complete
        // For now, return empty vec as placeholder
        Vec::new()
    }

    /// Decode WebSocket frame bytes to message
    ///
    /// Parses raw WebSocket frame bytes into a high-level Message
    /// according to RFC 6455.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let frame_bytes = vec![0x81, 0x05, b'H', b'e', b'l', b'l', b'o'];
    /// let msg = FrameHandler::decode(&frame_bytes);
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(Message)` if decoding succeeds
    /// - `None` if the frame is invalid or incomplete
    ///
    /// # Note
    ///
    /// This is a stub implementation. Actual decoding will be handled by
    /// hyper_tungstenite when the server layer is complete.
    pub fn decode(_data: &[u8]) -> Option<Message> {
        // TODO: Implement actual frame decoding when hyper integration is complete
        // For now, return None as placeholder
        None
    }

    /// Parse frame header
    ///
    /// Extracts frame metadata from the first 2+ bytes of a WebSocket frame.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = vec![0x81, 0x05]; // FIN=1, opcode=1 (text), len=5
    /// let (fin, opcode, masked, payload_len) = FrameHandler::parse_header(&data)?;
    /// ```
    pub fn parse_header(data: &[u8]) -> Option<(bool, Opcode, bool, u64)> {
        if data.len() < 2 {
            return None;
        }

        let byte1 = data[0];
        let byte2 = data[1];

        let fin = (byte1 & 0x80) != 0;
        let opcode = Opcode::from_u8(byte1)?;
        let masked = (byte2 & 0x80) != 0;
        let mut payload_len = (byte2 & 0x7F) as u64;

        let mut offset = 2;

        // Extended payload length
        if payload_len == 126 {
            if data.len() < offset + 2 {
                return None;
            }
            payload_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as u64;
            offset += 2;
        } else if payload_len == 127 {
            if data.len() < offset + 8 {
                return None;
            }
            payload_len = u64::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
        }

        Some((fin, opcode, masked, payload_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::from_u8(0x0), Some(Opcode::Continuation));
        assert_eq!(Opcode::from_u8(0x1), Some(Opcode::Text));
        assert_eq!(Opcode::from_u8(0x2), Some(Opcode::Binary));
        assert_eq!(Opcode::from_u8(0x8), Some(Opcode::Close));
        assert_eq!(Opcode::from_u8(0x9), Some(Opcode::Ping));
        assert_eq!(Opcode::from_u8(0xA), Some(Opcode::Pong));
        assert_eq!(Opcode::from_u8(0xF), None);
    }

    #[test]
    fn test_opcode_is_control() {
        assert!(!Opcode::Continuation.is_control());
        assert!(!Opcode::Text.is_control());
        assert!(!Opcode::Binary.is_control());
        assert!(Opcode::Close.is_control());
        assert!(Opcode::Ping.is_control());
        assert!(Opcode::Pong.is_control());
    }

    #[test]
    fn test_frame_new() {
        let frame = Frame::new(Opcode::Text, b"Hello".to_vec());
        assert!(frame.fin);
        assert_eq!(frame.opcode, Opcode::Text);
        assert_eq!(frame.payload, b"Hello");
    }

    #[test]
    fn test_frame_text() {
        let frame = Frame::text("Hello");
        assert_eq!(frame.opcode, Opcode::Text);
        assert_eq!(frame.payload, b"Hello");
    }

    #[test]
    fn test_frame_binary() {
        let frame = Frame::binary(vec![1, 2, 3]);
        assert_eq!(frame.opcode, Opcode::Binary);
        assert_eq!(frame.payload, vec![1, 2, 3]);
    }

    #[test]
    fn test_frame_ping() {
        let frame = Frame::ping(vec![1, 2]);
        assert_eq!(frame.opcode, Opcode::Ping);
        assert_eq!(frame.payload, vec![1, 2]);
    }

    #[test]
    fn test_frame_pong() {
        let frame = Frame::pong(vec![3, 4]);
        assert_eq!(frame.opcode, Opcode::Pong);
        assert_eq!(frame.payload, vec![3, 4]);
    }

    #[test]
    fn test_frame_close() {
        let frame = Frame::close();
        assert_eq!(frame.opcode, Opcode::Close);
        assert!(frame.payload.is_empty());
    }

    #[test]
    fn test_frame_apply_mask() {
        let mut frame = Frame::text("Hello");
        let original = frame.payload.clone();

        frame.apply_mask([1, 2, 3, 4]);

        assert!(frame.is_masked());
        assert_ne!(frame.payload, original);
    }

    #[test]
    fn test_frame_unmask() {
        let mut frame = Frame::text("Hello");
        let original = frame.payload.clone();

        frame.apply_mask([1, 2, 3, 4]);
        frame.unmask();

        assert!(!frame.is_masked());
        assert_eq!(frame.payload, original);
    }

    #[test]
    fn test_frame_is_control() {
        assert!(!Frame::text("Hello").is_control());
        assert!(!Frame::binary(vec![1, 2, 3]).is_control());
        assert!(Frame::ping(vec![]).is_control());
        assert!(Frame::pong(vec![]).is_control());
        assert!(Frame::close().is_control());
    }

    #[test]
    fn test_frame_payload_len() {
        let frame = Frame::text("Hello");
        assert_eq!(frame.payload_len(), 5);

        let frame = Frame::binary(vec![1, 2, 3]);
        assert_eq!(frame.payload_len(), 3);
    }

    #[test]
    fn test_parse_header_simple() {
        // FIN=1, opcode=1 (text), mask=0, len=5
        let data = vec![0x81, 0x05];
        let result = FrameHandler::parse_header(&data);

        assert!(result.is_some());
        let (fin, opcode, masked, payload_len) = result.unwrap();
        assert!(fin);
        assert_eq!(opcode, Opcode::Text);
        assert!(!masked);
        assert_eq!(payload_len, 5);
    }

    #[test]
    fn test_parse_header_masked() {
        // FIN=1, opcode=1 (text), mask=1, len=5
        let data = vec![0x81, 0x85];
        let result = FrameHandler::parse_header(&data);

        assert!(result.is_some());
        let (fin, opcode, masked, payload_len) = result.unwrap();
        assert!(fin);
        assert_eq!(opcode, Opcode::Text);
        assert!(masked);
        assert_eq!(payload_len, 5);
    }

    #[test]
    fn test_parse_header_extended_16bit() {
        // FIN=1, opcode=1 (text), mask=0, len=126 (extended 16-bit)
        let data = vec![0x81, 0x7E, 0x01, 0x00]; // len = 256
        let result = FrameHandler::parse_header(&data);

        assert!(result.is_some());
        let (_, _, _, payload_len) = result.unwrap();
        assert_eq!(payload_len, 256);
    }

    #[test]
    fn test_parse_header_extended_64bit() {
        // FIN=1, opcode=1 (text), mask=0, len=127 (extended 64-bit)
        let data = vec![0x81, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]; // len = 65536
        let result = FrameHandler::parse_header(&data);

        assert!(result.is_some());
        let (_, _, _, payload_len) = result.unwrap();
        assert_eq!(payload_len, 65536);
    }

    #[test]
    fn test_parse_header_incomplete() {
        let data = vec![0x81]; // Only 1 byte
        assert!(FrameHandler::parse_header(&data).is_none());
    }

    #[test]
    fn test_parse_header_control_frame() {
        // FIN=1, opcode=9 (ping), mask=0, len=0
        let data = vec![0x89, 0x00];
        let result = FrameHandler::parse_header(&data);

        assert!(result.is_some());
        let (fin, opcode, masked, payload_len) = result.unwrap();
        assert!(fin);
        assert_eq!(opcode, Opcode::Ping);
        assert!(!masked);
        assert_eq!(payload_len, 0);
    }

    #[test]
    fn test_frame_default() {
        let frame = Frame::default();
        assert!(frame.fin);
        assert!(!frame.rsv1);
        assert!(!frame.rsv2);
        assert!(!frame.rsv3);
        assert_eq!(frame.opcode, Opcode::Text);
        assert!(frame.mask.is_none());
        assert!(frame.payload.is_empty());
    }

    #[test]
    fn test_frame_equality() {
        let frame1 = Frame::text("Hello");
        let frame2 = Frame::text("Hello");
        let frame3 = Frame::text("World");

        assert_eq!(frame1, frame2);
        assert_ne!(frame1, frame3);
    }

    #[test]
    fn test_frame_clone() {
        let frame = Frame::text("Hello");
        let cloned = frame.clone();
        assert_eq!(frame, cloned);
    }
}
