//! SSE sender

use crate::upgrade::sse::Event;
use tokio::sync::mpsc;

/// SSE sender
#[derive(Clone)]
pub struct SseSender {
    tx: mpsc::UnboundedSender<Event>,
}

impl SseSender {
    /// Create new sender
    pub(crate) fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self { tx }
    }

    /// Send event
    pub async fn send(&self, event: Event) -> Result<(), mpsc::error::SendError<Event>> {
        self.tx.send(event)
    }

    /// Send data event
    pub async fn send_data(
        &self,
        data: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<Event>> {
        self.send(Event::data(data)).await
    }
}
