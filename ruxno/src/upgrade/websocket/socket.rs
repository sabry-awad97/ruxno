//! WebSocket connection
//!
//! Provides a high-level abstraction over WebSocket connections for sending
//! and receiving messages.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::upgrade::websocket::{WebSocket, Message};
//!
//! async fn handle_websocket(mut socket: WebSocket) {
//!     // Receive messages
//!     while let Some(Ok(msg)) = socket.recv().await {
//!         match msg {
//!             Message::Text(text) => {
//!                 println!("Received: {}", text);
//!                 socket.send(Message::Text(text)).await.ok();
//!             }
//!             Message::Binary(data) => {
//!                 println!("Received {} bytes", data.len());
//!                 socket.send(Message::Binary(data)).await.ok();
//!             }
//!             Message::Ping(data) => {
//!                 socket.send(Message::Pong(data)).await.ok();
//!             }
//!             Message::Close => break,
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use crate::core::CoreError;
use crate::upgrade::websocket::Message;
use std::sync::Arc;
use tokio::sync::Mutex;

/// WebSocket connection
///
/// Wraps a WebSocket stream and provides high-level methods for sending
/// and receiving messages. Handles ping/pong frames automatically.
///
/// # Design
///
/// The WebSocket is wrapped in `Arc<Mutex<>>` to allow sharing across tasks
/// while maintaining exclusive access for send/recv operations.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::{WebSocket, Message};
///
/// async fn echo_server(mut socket: WebSocket) {
///     while let Some(Ok(msg)) = socket.recv().await {
///         if let Message::Text(text) = msg {
///             socket.send(Message::Text(text)).await.ok();
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct WebSocket {
    /// Inner WebSocket stream (placeholder for future implementation)
    inner: Arc<Mutex<WebSocketInner>>,
}

/// Inner WebSocket state
struct WebSocketInner {
    /// Closed flag
    closed: bool,
}

impl WebSocket {
    /// Create a new WebSocket connection
    ///
    /// This is an internal constructor used by the upgrade process.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let socket = WebSocket::new();
    /// ```
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(WebSocketInner { closed: false })),
        }
    }

    /// Send a message
    ///
    /// Sends a message over the WebSocket connection. Supports text, binary,
    /// ping, pong, and close messages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::upgrade::websocket::Message;
    ///
    /// // Send text message
    /// socket.send(Message::Text("Hello".to_string())).await?;
    ///
    /// // Send binary message
    /// socket.send(Message::Binary(bytes)).await?;
    ///
    /// // Send ping
    /// socket.send(Message::Ping(vec![])).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the connection is closed or if sending fails.
    pub async fn send(&mut self, msg: Message) -> Result<(), CoreError> {
        let mut inner = self.inner.lock().await;

        if inner.closed {
            return Err(CoreError::internal("WebSocket connection is closed"));
        }

        // TODO: Implement actual message sending when hyper integration is complete
        // For now, just validate the message type
        match msg {
            Message::Text(_) | Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => Ok(()),
            Message::Close => {
                inner.closed = true;
                Ok(())
            }
        }
    }

    /// Send a text message
    ///
    /// Convenience method for sending text messages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// socket.send_text("Hello, World!").await?;
    /// ```
    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<(), CoreError> {
        self.send(Message::text(text)).await
    }

    /// Send a binary message
    ///
    /// Convenience method for sending binary messages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// socket.send_binary(vec![1, 2, 3, 4]).await?;
    /// ```
    pub async fn send_binary(&mut self, data: impl Into<bytes::Bytes>) -> Result<(), CoreError> {
        self.send(Message::binary(data)).await
    }

    /// Send a ping message
    ///
    /// Sends a ping frame. The peer should respond with a pong.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// socket.send_ping(vec![]).await?;
    /// ```
    pub async fn send_ping(&mut self, data: Vec<u8>) -> Result<(), CoreError> {
        self.send(Message::ping(data)).await
    }

    /// Send a pong message
    ///
    /// Sends a pong frame, typically in response to a ping.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// socket.send_pong(vec![]).await?;
    /// ```
    pub async fn send_pong(&mut self, data: Vec<u8>) -> Result<(), CoreError> {
        self.send(Message::pong(data)).await
    }

    /// Receive a message
    ///
    /// Receives the next message from the WebSocket connection.
    /// Returns `None` when the connection is closed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// while let Some(Ok(msg)) = socket.recv().await {
    ///     match msg {
    ///         Message::Text(text) => println!("Received: {}", text),
    ///         Message::Binary(data) => println!("Received {} bytes", data.len()),
    ///         Message::Close => break,
    ///         _ => {}
    ///     }
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(Ok(Message))` - A message was received
    /// - `Some(Err(CoreError))` - An error occurred
    /// - `None` - The connection is closed
    pub async fn recv(&mut self) -> Option<Result<Message, CoreError>> {
        let inner = self.inner.lock().await;

        if inner.closed {
            return None;
        }

        // TODO: Implement actual message receiving when hyper integration is complete
        // For now, return None to indicate no messages
        None
    }

    /// Close the connection
    ///
    /// Sends a close frame and closes the WebSocket connection gracefully.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// socket.close().await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if closing fails.
    pub async fn close(mut self) -> Result<(), CoreError> {
        self.send(Message::Close).await
    }

    /// Check if the connection is closed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if socket.is_closed().await {
    ///     println!("Connection closed");
    /// }
    /// ```
    pub async fn is_closed(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_websocket_new() {
        let socket = WebSocket::new();
        assert!(!socket.is_closed().await);
    }

    #[tokio::test]
    async fn test_send_text() {
        let mut socket = WebSocket::new();
        let result = socket.send_text("Hello").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_binary() {
        let mut socket = WebSocket::new();
        let result = socket.send_binary(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_ping() {
        let mut socket = WebSocket::new();
        let result = socket.send_ping(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_pong() {
        let mut socket = WebSocket::new();
        let result = socket.send_pong(vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_after_close() {
        let mut socket = WebSocket::new();
        socket.send(Message::Close).await.unwrap();

        let result = socket.send_text("Hello").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_recv_on_closed() {
        let mut socket = WebSocket::new();
        socket.send(Message::Close).await.unwrap();

        let result = socket.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_closed() {
        let mut socket = WebSocket::new();
        assert!(!socket.is_closed().await);

        socket.send(Message::Close).await.unwrap();
        assert!(socket.is_closed().await);
    }

    #[tokio::test]
    async fn test_clone() {
        let socket = WebSocket::new();
        let socket2 = socket.clone();

        assert!(!socket.is_closed().await);
        assert!(!socket2.is_closed().await);
    }

    #[tokio::test]
    async fn test_send_all_message_types() {
        let mut socket = WebSocket::new();

        // Test all message types
        assert!(socket.send(Message::text("test")).await.is_ok());
        assert!(socket
            .send(Message::binary(Bytes::from(vec![1, 2, 3])))
            .await
            .is_ok());
        assert!(socket.send(Message::ping(vec![1, 2])).await.is_ok());
        assert!(socket.send(Message::pong(vec![3, 4])).await.is_ok());
        assert!(socket.send(Message::close()).await.is_ok());

        // After close, should be closed
        assert!(socket.is_closed().await);
    }
}
