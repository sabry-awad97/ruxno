//! SSE stream

use crate::domain::Response;
use crate::upgrade::sse::{Event, SseSender};
use tokio::sync::mpsc;

/// SSE stream
pub struct SseStream {
    tx: mpsc::UnboundedSender<Event>,
}

impl SseStream {
    /// Create new SSE stream
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();
        Self { tx }
    }

    /// Get sender
    pub fn sender(&self) -> SseSender {
        SseSender::new(self.tx.clone())
    }

    /// Convert to response
    pub fn into_response(self) -> Response {
        // TODO: Create streaming response
        todo!("Implement SseStream::into_response")
    }
}

impl Default for SseStream {
    fn default() -> Self {
        Self::new()
    }
}
