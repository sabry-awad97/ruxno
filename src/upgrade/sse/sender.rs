//! SSE sender
//!
//! This module provides the `SseSender` for sending Server-Sent Events to clients.
//! The sender uses an unbounded channel to queue events for transmission.
//!
//! # Example
//!
//! ```rust
//! use ruxno_clean::upgrade::sse::{SseStream, Event};
//!
//! # async fn example() {
//! let stream = SseStream::new();
//! let sender = stream.sender();
//!
//! // Send simple data
//! sender.send_data("Hello, world!").await.ok();
//!
//! // Send event with type and ID
//! sender.send(Event::new("User joined").with_event("user-join").with_id("1")).await.ok();
//! # }
//! ```

use crate::upgrade::sse::Event;
use tokio::sync::mpsc;

/// SSE sender for transmitting events to clients
///
/// The sender is cloneable and can be shared across tasks. All clones send
/// to the same underlying channel, allowing multiple producers to send events
/// to a single SSE stream.
#[derive(Clone)]
pub struct SseSender {
    tx: mpsc::UnboundedSender<Event>,
}

impl SseSender {
    /// Create new sender (internal use)
    ///
    /// This is typically called by `SseStream::sender()` rather than directly.
    pub(crate) fn new(tx: mpsc::UnboundedSender<Event>) -> Self {
        Self { tx }
    }

    /// Send event to the client
    ///
    /// Queues an event for transmission. The event will be formatted according
    /// to the SSE specification and sent to the client.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the receiver has been dropped (client disconnected).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::{SseStream, Event};
    ///
    /// # async fn example() {
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    ///
    /// let event = Event::new("Hello").with_event("greeting");
    /// sender.send(event).await.ok();
    /// # }
    /// ```
    pub async fn send(&self, event: Event) -> Result<(), mpsc::error::SendError<Event>> {
        self.tx.send(event)
    }

    /// Send simple data event
    ///
    /// Convenience method for sending a data-only event without event type or ID.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the receiver has been dropped (client disconnected).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// # async fn example() {
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    ///
    /// sender.send_data("Simple message").await.ok();
    /// # }
    /// ```
    pub async fn send_data(
        &self,
        data: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<Event>> {
        self.send(Event::data(data)).await
    }

    /// Send event with type
    ///
    /// Convenience method for sending an event with a specific event type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// # async fn example() {
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    ///
    /// sender.send_event("user-join", "Alice joined the chat").await.ok();
    /// # }
    /// ```
    pub async fn send_event(
        &self,
        event_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<Event>> {
        self.send(Event::new(data).with_event(event_type)).await
    }

    /// Check if the receiver is still active
    ///
    /// Returns `true` if the client is still connected and can receive events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    ///
    /// assert!(sender.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        !self.tx.is_closed()
    }

    /// Get the number of queued events
    ///
    /// Returns the number of events waiting to be sent. This is always 0 for
    /// unbounded channels since events are sent immediately.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    ///
    /// // Unbounded channels don't queue
    /// assert_eq!(sender.queued_count(), 0);
    /// ```
    pub fn queued_count(&self) -> usize {
        // Unbounded channels don't expose queue length
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sender_send() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = SseSender::new(tx);

        let event = Event::new("test data");
        sender.send(event.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn test_sender_send_data() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = SseSender::new(tx);

        sender.send_data("Hello").await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.data, "Hello");
        assert_eq!(received.event, None);
    }

    #[tokio::test]
    async fn test_sender_send_event() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = SseSender::new(tx);

        sender.send_event("greeting", "Hello").await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.data, "Hello");
        assert_eq!(received.event, Some("greeting".to_string()));
    }

    #[tokio::test]
    async fn test_sender_multiple_events() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = SseSender::new(tx);

        sender.send_data("Event 1").await.unwrap();
        sender.send_data("Event 2").await.unwrap();
        sender.send_data("Event 3").await.unwrap();

        assert_eq!(rx.recv().await.unwrap().data, "Event 1");
        assert_eq!(rx.recv().await.unwrap().data, "Event 2");
        assert_eq!(rx.recv().await.unwrap().data, "Event 3");
    }

    #[tokio::test]
    async fn test_sender_disconnected() {
        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let sender = SseSender::new(tx);

        // Drop receiver
        drop(rx);

        // Send should fail
        let result = sender.send_data("test").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sender_is_connected() {
        let (tx, _rx) = mpsc::unbounded_channel::<Event>();
        let sender = SseSender::new(tx);

        assert!(sender.is_connected());
    }

    #[test]
    fn test_sender_is_disconnected() {
        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let sender = SseSender::new(tx);

        drop(rx);

        assert!(!sender.is_connected());
    }

    #[test]
    fn test_sender_clone() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender1 = SseSender::new(tx);
        let sender2 = sender1.clone();

        // Both senders work
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            sender1.send_data("From sender 1").await.unwrap();
            sender2.send_data("From sender 2").await.unwrap();

            assert_eq!(rx.recv().await.unwrap().data, "From sender 1");
            assert_eq!(rx.recv().await.unwrap().data, "From sender 2");
        });
    }

    #[test]
    fn test_sender_queued_count() {
        let (tx, _rx) = mpsc::unbounded_channel::<Event>();
        let sender = SseSender::new(tx);

        // Unbounded channels always return 0
        assert_eq!(sender.queued_count(), 0);
    }

    #[test]
    fn test_sender_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SseSender>();
        assert_sync::<SseSender>();
    }
}
