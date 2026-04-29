//! WebSocket message types
//!
//! Provides type-safe WebSocket message types using the newtype pattern
//! for better domain modeling and type safety.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::upgrade::websocket::{Message, TextMessage, BinaryMessage};
//!
//! // Create messages using newtypes
//! let text = TextMessage::new("Hello, World!");
//! let msg = Message::from(text);
//!
//! // Create messages directly
//! let msg = Message::text("Hello");
//! let msg = Message::binary(vec![1, 2, 3]);
//!
//! // Pattern matching
//! match msg {
//!     Message::Text(text) => println!("Text: {}", text.as_str()),
//!     Message::Binary(data) => println!("Binary: {} bytes", data.len()),
//!     _ => {}
//! }
//! ```

use bytes::Bytes;

/// WebSocket text message
///
/// Newtype wrapper for text messages providing type safety and
/// domain-specific operations.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::TextMessage;
///
/// let msg = TextMessage::new("Hello, World!");
/// assert_eq!(msg.as_str(), "Hello, World!");
/// assert_eq!(msg.len(), 13);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextMessage(String);

impl TextMessage {
    /// Create a new text message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = TextMessage::new("Hello");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Get the text as a string slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = TextMessage::new("Hello");
    /// assert_eq!(msg.as_str(), "Hello");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the text in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = TextMessage::new("Hello");
    /// assert_eq!(msg.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the text is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = TextMessage::new("");
    /// assert!(msg.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and return the inner string
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = TextMessage::new("Hello");
    /// let text = msg.into_string();
    /// ```
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for TextMessage {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl From<&str> for TextMessage {
    fn from(text: &str) -> Self {
        Self(text.to_string())
    }
}

impl AsRef<str> for TextMessage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TextMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// WebSocket binary message
///
/// Newtype wrapper for binary messages providing type safety and
/// domain-specific operations.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::BinaryMessage;
///
/// let msg = BinaryMessage::new(vec![1, 2, 3]);
/// assert_eq!(msg.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryMessage(Bytes);

impl BinaryMessage {
    /// Create a new binary message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = BinaryMessage::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self(data.into())
    }

    /// Get the data as a byte slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = BinaryMessage::new(vec![1, 2, 3]);
    /// assert_eq!(msg.as_bytes(), &[1, 2, 3]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Get the length of the data in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = BinaryMessage::new(vec![1, 2, 3]);
    /// assert_eq!(msg.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the data is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = BinaryMessage::new(vec![]);
    /// assert!(msg.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and return the inner bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = BinaryMessage::new(vec![1, 2, 3]);
    /// let bytes = msg.into_bytes();
    /// ```
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

impl From<Vec<u8>> for BinaryMessage {
    fn from(data: Vec<u8>) -> Self {
        Self(Bytes::from(data))
    }
}

impl From<Bytes> for BinaryMessage {
    fn from(data: Bytes) -> Self {
        Self(data)
    }
}

impl From<&[u8]> for BinaryMessage {
    fn from(data: &[u8]) -> Self {
        Self(Bytes::copy_from_slice(data))
    }
}

impl AsRef<[u8]> for BinaryMessage {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// WebSocket control frame data
///
/// Newtype wrapper for control frame payloads (ping/pong).
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::ControlData;
///
/// let data = ControlData::new(vec![1, 2, 3]);
/// assert_eq!(data.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlData(Vec<u8>);

impl ControlData {
    /// Create new control data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Create empty control data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::empty();
    /// assert!(data.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Get the data as a slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::new(vec![1, 2, 3]);
    /// assert_eq!(data.as_slice(), &[1, 2, 3]);
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Get the length of the data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::new(vec![1, 2, 3]);
    /// assert_eq!(data.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the data is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::empty();
    /// assert!(data.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and return the inner vector
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = ControlData::new(vec![1, 2, 3]);
    /// let vec = data.into_vec();
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for ControlData {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<&[u8]> for ControlData {
    fn from(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}

impl AsRef<[u8]> for ControlData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// WebSocket message
///
/// Represents all possible WebSocket message types following RFC 6455.
/// Uses newtypes for type safety and better domain modeling.
///
/// # Message Types
///
/// - **Text**: UTF-8 encoded text messages
/// - **Binary**: Raw binary data
/// - **Ping**: Control frame for connection keep-alive
/// - **Pong**: Response to ping frames
/// - **Close**: Connection close frame
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::Message;
///
/// // Create messages
/// let text = Message::text("Hello");
/// let binary = Message::binary(vec![1, 2, 3]);
/// let ping = Message::ping(vec![]);
/// let pong = Message::pong(vec![]);
/// let close = Message::close();
///
/// // Pattern matching
/// match text {
///     Message::Text(msg) => println!("Text: {}", msg.as_str()),
///     Message::Binary(msg) => println!("Binary: {} bytes", msg.len()),
///     Message::Ping(data) => println!("Ping: {} bytes", data.len()),
///     Message::Pong(data) => println!("Pong: {} bytes", data.len()),
///     Message::Close => println!("Close"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Text message (UTF-8 encoded)
    Text(TextMessage),
    /// Binary message (raw bytes)
    Binary(BinaryMessage),
    /// Ping control frame
    Ping(ControlData),
    /// Pong control frame
    Pong(ControlData),
    /// Close control frame
    Close,
}

impl Message {
    /// Create a text message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::text("Hello, World!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextMessage::new(text))
    }

    /// Create a binary message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::binary(vec![1, 2, 3]);
    /// ```
    pub fn binary(data: impl Into<Bytes>) -> Self {
        Self::Binary(BinaryMessage::new(data))
    }

    /// Create a ping message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::ping(vec![1, 2, 3]);
    /// ```
    pub fn ping(data: Vec<u8>) -> Self {
        Self::Ping(ControlData::new(data))
    }

    /// Create a pong message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::pong(vec![1, 2, 3]);
    /// ```
    pub fn pong(data: Vec<u8>) -> Self {
        Self::Pong(ControlData::new(data))
    }

    /// Create a close message
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::close();
    /// ```
    pub fn close() -> Self {
        Self::Close
    }

    /// Check if message is text
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::text("Hello");
    /// assert!(msg.is_text());
    /// ```
    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    /// Check if message is binary
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::binary(vec![1, 2, 3]);
    /// assert!(msg.is_binary());
    /// ```
    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    /// Check if message is ping
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::ping(vec![]);
    /// assert!(msg.is_ping());
    /// ```
    pub fn is_ping(&self) -> bool {
        matches!(self, Message::Ping(_))
    }

    /// Check if message is pong
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::pong(vec![]);
    /// assert!(msg.is_pong());
    /// ```
    pub fn is_pong(&self) -> bool {
        matches!(self, Message::Pong(_))
    }

    /// Check if message is close
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::close();
    /// assert!(msg.is_close());
    /// ```
    pub fn is_close(&self) -> bool {
        matches!(self, Message::Close)
    }

    /// Check if message is a control frame (ping, pong, or close)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let msg = Message::ping(vec![]);
    /// assert!(msg.is_control());
    /// ```
    pub fn is_control(&self) -> bool {
        matches!(self, Message::Ping(_) | Message::Pong(_) | Message::Close)
    }
}

// Conversions from newtypes to Message

impl From<TextMessage> for Message {
    fn from(msg: TextMessage) -> Self {
        Self::Text(msg)
    }
}

impl From<BinaryMessage> for Message {
    fn from(msg: BinaryMessage) -> Self {
        Self::Binary(msg)
    }
}

// Conversions from primitives to Message

impl From<String> for Message {
    fn from(text: String) -> Self {
        Self::text(text)
    }
}

impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Self::text(text)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Self::binary(data)
    }
}

impl From<Bytes> for Message {
    fn from(data: Bytes) -> Self {
        Self::binary(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TextMessage tests
    #[test]
    fn test_text_message_new() {
        let msg = TextMessage::new("Hello");
        assert_eq!(msg.as_str(), "Hello");
    }

    #[test]
    fn test_text_message_len() {
        let msg = TextMessage::new("Hello");
        assert_eq!(msg.len(), 5);
    }

    #[test]
    fn test_text_message_is_empty() {
        let msg = TextMessage::new("");
        assert!(msg.is_empty());

        let msg = TextMessage::new("Hello");
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_text_message_from_string() {
        let msg = TextMessage::from("Hello".to_string());
        assert_eq!(msg.as_str(), "Hello");
    }

    #[test]
    fn test_text_message_from_str() {
        let msg = TextMessage::from("Hello");
        assert_eq!(msg.as_str(), "Hello");
    }

    #[test]
    fn test_text_message_display() {
        let msg = TextMessage::new("Hello");
        assert_eq!(format!("{}", msg), "Hello");
    }

    // BinaryMessage tests
    #[test]
    fn test_binary_message_new() {
        let msg = BinaryMessage::new(vec![1, 2, 3]);
        assert_eq!(msg.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_binary_message_len() {
        let msg = BinaryMessage::new(vec![1, 2, 3]);
        assert_eq!(msg.len(), 3);
    }

    #[test]
    fn test_binary_message_is_empty() {
        let msg = BinaryMessage::new(vec![]);
        assert!(msg.is_empty());

        let msg = BinaryMessage::new(vec![1, 2, 3]);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_binary_message_from_vec() {
        let msg = BinaryMessage::from(vec![1, 2, 3]);
        assert_eq!(msg.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_binary_message_from_bytes() {
        let msg = BinaryMessage::from(Bytes::from(vec![1, 2, 3]));
        assert_eq!(msg.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_binary_message_from_slice() {
        let msg = BinaryMessage::from(&[1, 2, 3][..]);
        assert_eq!(msg.as_bytes(), &[1, 2, 3]);
    }

    // ControlData tests
    #[test]
    fn test_control_data_new() {
        let data = ControlData::new(vec![1, 2, 3]);
        assert_eq!(data.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_control_data_empty() {
        let data = ControlData::empty();
        assert!(data.is_empty());
    }

    #[test]
    fn test_control_data_len() {
        let data = ControlData::new(vec![1, 2, 3]);
        assert_eq!(data.len(), 3);
    }

    // Message tests
    #[test]
    fn test_message_text() {
        let msg = Message::text("Hello");
        assert!(msg.is_text());
        assert!(!msg.is_binary());
        assert!(!msg.is_control());
    }

    #[test]
    fn test_message_binary() {
        let msg = Message::binary(vec![1, 2, 3]);
        assert!(msg.is_binary());
        assert!(!msg.is_text());
        assert!(!msg.is_control());
    }

    #[test]
    fn test_message_ping() {
        let msg = Message::ping(vec![]);
        assert!(msg.is_ping());
        assert!(msg.is_control());
        assert!(!msg.is_text());
    }

    #[test]
    fn test_message_pong() {
        let msg = Message::pong(vec![]);
        assert!(msg.is_pong());
        assert!(msg.is_control());
        assert!(!msg.is_binary());
    }

    #[test]
    fn test_message_close() {
        let msg = Message::close();
        assert!(msg.is_close());
        assert!(msg.is_control());
    }

    #[test]
    fn test_message_from_string() {
        let msg = Message::from("Hello".to_string());
        assert!(msg.is_text());
    }

    #[test]
    fn test_message_from_str() {
        let msg = Message::from("Hello");
        assert!(msg.is_text());
    }

    #[test]
    fn test_message_from_vec() {
        let msg = Message::from(vec![1, 2, 3]);
        assert!(msg.is_binary());
    }

    #[test]
    fn test_message_from_bytes() {
        let msg = Message::from(Bytes::from(vec![1, 2, 3]));
        assert!(msg.is_binary());
    }

    #[test]
    fn test_message_equality() {
        let msg1 = Message::text("Hello");
        let msg2 = Message::text("Hello");
        let msg3 = Message::text("World");

        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_message_clone() {
        let msg = Message::text("Hello");
        let cloned = msg.clone();
        assert_eq!(msg, cloned);
    }

    #[test]
    fn test_text_message_equality() {
        let msg1 = TextMessage::new("Hello");
        let msg2 = TextMessage::new("Hello");
        let msg3 = TextMessage::new("World");

        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_binary_message_equality() {
        let msg1 = BinaryMessage::new(vec![1, 2, 3]);
        let msg2 = BinaryMessage::new(vec![1, 2, 3]);
        let msg3 = BinaryMessage::new(vec![4, 5, 6]);

        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }
}
