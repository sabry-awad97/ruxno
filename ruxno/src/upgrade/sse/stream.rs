//! SSE stream
//!
//! This module provides the `SseStream` for creating Server-Sent Events responses.
//! SSE is a standard for pushing real-time updates from server to client over HTTP.
//!
//! # Example
//!
//! ```rust
//! use ruxno_clean::upgrade::sse::SseStream;
//! use ruxno_clean::domain::Context;
//!
//! # async fn handler(ctx: Context) -> Result<ruxno_clean::domain::Response, ruxno_clean::core::CoreError> {
//! // Create SSE stream
//! let stream = SseStream::new();
//! let sender = stream.sender();
//!
//! // Spawn task to send events
//! tokio::spawn(async move {
//!     sender.send_data("Event 1").await.ok();
//!     sender.send_data("Event 2").await.ok();
//! });
//!
//! // Return streaming response
//! Ok(stream.into_response())
//! # }
//! ```

use crate::core::StatusCode;
use crate::domain::Response;
use crate::http::Body;
use crate::upgrade::sse::{Event, SseSender};
use bytes::Bytes;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// SSE stream for sending Server-Sent Events to clients
///
/// Creates a streaming HTTP response with `Content-Type: text/event-stream`.
/// Events are sent through a channel and formatted according to the SSE specification.
pub struct SseStream {
    tx: mpsc::UnboundedSender<Event>,
    rx: Option<mpsc::UnboundedReceiver<Event>>,
}

impl SseStream {
    /// Create new SSE stream
    ///
    /// Creates a new stream with an unbounded channel for sending events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// let stream = SseStream::new();
    /// let sender = stream.sender();
    /// ```
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx: Some(rx) }
    }

    /// Get sender for this stream
    ///
    /// Returns a cloneable sender that can be used to send events to the stream.
    /// Multiple senders can be created and used concurrently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// let stream = SseStream::new();
    /// let sender1 = stream.sender();
    /// let sender2 = stream.sender();
    /// ```
    pub fn sender(&self) -> SseSender {
        SseSender::new(self.tx.clone())
    }

    /// Convert stream into HTTP response
    ///
    /// Creates a streaming response with proper SSE headers:
    /// - `Content-Type: text/event-stream`
    /// - `Cache-Control: no-cache`
    /// - `Connection: keep-alive`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::SseStream;
    ///
    /// let stream = SseStream::new();
    /// let response = stream.into_response();
    /// ```
    pub fn into_response(mut self) -> Response {
        let rx = self.rx.take().expect("receiver already taken");
        let event_stream = EventStream::new(rx);

        // Create streaming body
        let body = Body::from_stream(event_stream);

        // Create response with SSE headers and streaming body
        Response::new()
            .with_status(StatusCode::OK)
            .with_header("Content-Type", "text/event-stream")
            .with_header("Cache-Control", "no-cache")
            .with_header("Connection", "keep-alive")
            .with_header("X-Accel-Buffering", "no") // Disable nginx buffering
            .with_stream(body.into_stream())
    }
}

impl Default for SseStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal stream adapter for converting Events to Bytes
struct EventStream {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventStream {
    fn new(rx: mpsc::UnboundedReceiver<Event>) -> Self {
        Self { rx }
    }
}

impl Stream for EventStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(event)) => {
                let formatted: String = event.format();
                Poll::Ready(Some(Ok(Bytes::from(formatted))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_new() {
        let stream = SseStream::new();
        let _sender = stream.sender();
    }

    #[test]
    fn test_stream_default() {
        let stream = SseStream::default();
        let _sender = stream.sender();
    }

    #[test]
    fn test_stream_sender() {
        let stream = SseStream::new();
        let sender1 = stream.sender();
        let sender2 = stream.sender();

        assert!(sender1.is_connected());
        assert!(sender2.is_connected());
    }

    #[tokio::test]
    async fn test_stream_into_response() {
        let stream = SseStream::new();
        let response = stream.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        // Check headers
        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
        assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
        assert_eq!(headers.get("connection").unwrap(), "keep-alive");
    }

    #[tokio::test]
    async fn test_stream_send_and_receive() {
        let stream = SseStream::new();
        let sender = stream.sender();

        // Send events before converting to response
        sender.send_data("Event 1").await.unwrap();
        sender.send_data("Event 2").await.unwrap();

        // Note: In real usage, events would be sent after response is created
        // This test just verifies the channel works
        assert!(sender.is_connected());
    }

    #[tokio::test]
    async fn test_event_stream_formatting() {
        use futures_util::StreamExt;

        let (tx, rx) = mpsc::unbounded_channel();
        let mut event_stream = EventStream::new(rx);

        // Send event
        tx.send(Event::new("test data")).unwrap();

        // Receive formatted event
        let item = event_stream.next().await.unwrap().unwrap();
        let formatted = String::from_utf8(item.to_vec()).unwrap();

        assert_eq!(formatted, "data: test data\n\n");
    }

    #[tokio::test]
    async fn test_event_stream_multiple_events() {
        use futures_util::StreamExt;

        let (tx, rx) = mpsc::unbounded_channel();
        let mut event_stream = EventStream::new(rx);

        // Send multiple events
        tx.send(Event::new("Event 1")).unwrap();
        tx.send(Event::new("Event 2").with_event("custom")).unwrap();
        tx.send(Event::new("Event 3").with_id("123")).unwrap();

        // Receive all events
        let item1 = event_stream.next().await.unwrap().unwrap();
        let item2 = event_stream.next().await.unwrap().unwrap();
        let item3 = event_stream.next().await.unwrap().unwrap();

        assert!(
            String::from_utf8(item1.to_vec())
                .unwrap()
                .contains("Event 1")
        );
        assert!(
            String::from_utf8(item2.to_vec())
                .unwrap()
                .contains("custom")
        );
        assert!(String::from_utf8(item3.to_vec()).unwrap().contains("123"));
    }

    #[tokio::test]
    async fn test_event_stream_closed() {
        use futures_util::StreamExt;

        let (tx, rx) = mpsc::unbounded_channel();
        let mut event_stream = EventStream::new(rx);

        // Close sender
        drop(tx);

        // Stream should end
        assert!(event_stream.next().await.is_none());
    }

    #[test]
    fn test_stream_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SseStream>();
    }
}
