//! WebSocket connection

use crate::core::CoreError;
use crate::upgrade::websocket::Message;

/// WebSocket connection
pub struct WebSocket {
    // TODO: Add WebSocket stream
}

impl WebSocket {
    /// Send a message
    pub async fn send(&mut self, msg: Message) -> Result<(), CoreError> {
        // TODO: Send message
        todo!("Implement WebSocket::send")
    }

    /// Receive a message
    pub async fn recv(&mut self) -> Option<Result<Message, CoreError>> {
        // TODO: Receive message
        todo!("Implement WebSocket::recv")
    }

    /// Close the connection
    pub async fn close(self) -> Result<(), CoreError> {
        // TODO: Close connection
        todo!("Implement WebSocket::close")
    }
}
