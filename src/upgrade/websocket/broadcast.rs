//! WebSocket broadcasting

use crate::upgrade::websocket::Message;
use tokio::sync::broadcast;

/// Broadcaster for WebSocket messages
pub struct Broadcaster {
    tx: broadcast::Sender<Message>,
}

impl Broadcaster {
    /// Create new broadcaster
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Broadcast message
    pub fn broadcast(&self, msg: Message) -> Result<(), broadcast::error::SendError<Message>> {
        self.tx.send(msg).map(|_| ())
    }

    /// Subscribe to broadcasts
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }
}
