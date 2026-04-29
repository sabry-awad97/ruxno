//! SSE event formatting
//!
//! This module implements Server-Sent Events (SSE) event formatting according to the
//! [W3C Server-Sent Events specification](https://html.spec.whatwg.org/multipage/server-sent-events.html).
//!
//! # SSE Format
//!
//! Events are formatted as UTF-8 text with the following fields:
//! - `event:` - Event type (optional, defaults to "message")
//! - `data:` - Event data (required, can be multi-line)
//! - `id:` - Event ID (optional, for reconnection)
//! - `retry:` - Reconnection time in milliseconds (optional)
//!
//! Each field is on its own line, and events are separated by blank lines.
//!
//! # Example
//!
//! ```rust
//! use ruxno_clean::upgrade::sse::Event;
//!
//! // Simple data event
//! let event = Event::new("Hello, world!");
//! assert_eq!(event.format(), "data: Hello, world!\n\n");
//!
//! // Event with type and ID
//! let event = Event::new("User joined")
//!     .with_event("user-join")
//!     .with_id("123");
//! assert_eq!(
//!     event.format(),
//!     "event: user-join\ndata: User joined\nid: 123\n\n"
//! );
//! ```

/// SSE event
///
/// Represents a Server-Sent Event with optional event type, ID, and retry configuration.
/// Events are formatted according to the SSE specification for transmission over HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Event type (optional, defaults to "message" on client side)
    pub event: Option<String>,
    /// Event data (required)
    pub data: String,
    /// Event ID (optional, used for reconnection)
    pub id: Option<String>,
    /// Retry time in milliseconds (optional)
    pub retry: Option<u64>,
}

impl Event {
    /// Create new event with data
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::new("Hello, world!");
    /// assert_eq!(event.data, "Hello, world!");
    /// ```
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
            id: None,
            retry: None,
        }
    }

    /// Create data event (alias for `new`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::data("Hello");
    /// assert_eq!(event.data, "Hello");
    /// ```
    pub fn data(data: impl Into<String>) -> Self {
        Self::new(data)
    }

    /// Set event type
    ///
    /// The event type is used by the client to distinguish different event types.
    /// If not set, the client will use "message" as the default type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::new("User joined").with_event("user-join");
    /// assert_eq!(event.event, Some("user-join".to_string()));
    /// ```
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set event ID
    ///
    /// The event ID is used by the client for reconnection. When reconnecting,
    /// the client sends the last received event ID in the `Last-Event-ID` header.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::new("Data").with_id("123");
    /// assert_eq!(event.id, Some("123".to_string()));
    /// ```
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set retry time in milliseconds
    ///
    /// Tells the client how long to wait before attempting to reconnect
    /// if the connection is lost.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::new("Data").with_retry(5000);
    /// assert_eq!(event.retry, Some(5000));
    /// ```
    pub fn with_retry(mut self, retry: u64) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Format event according to SSE specification
    ///
    /// Returns a string formatted according to the SSE protocol:
    /// - Each field is on its own line
    /// - Multi-line data is split into multiple `data:` fields
    /// - Event ends with a blank line (`\n\n`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let event = Event::new("Hello")
    ///     .with_event("greeting")
    ///     .with_id("1");
    ///
    /// assert_eq!(
    ///     event.format(),
    ///     "event: greeting\ndata: Hello\nid: 1\n\n"
    /// );
    /// ```
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Event type (optional)
        if let Some(ref event) = self.event {
            output.push_str("event: ");
            output.push_str(event);
            output.push('\n');
        }

        // Data (required, can be multi-line)
        for line in self.data.lines() {
            output.push_str("data: ");
            output.push_str(line);
            output.push('\n');
        }

        // Handle empty data
        if self.data.is_empty() {
            output.push_str("data: \n");
        }

        // Event ID (optional)
        if let Some(ref id) = self.id {
            output.push_str("id: ");
            output.push_str(id);
            output.push('\n');
        }

        // Retry (optional)
        if let Some(retry) = self.retry {
            output.push_str("retry: ");
            output.push_str(&retry.to_string());
            output.push('\n');
        }

        // End with blank line
        output.push('\n');

        output
    }

    /// Create a comment event (for keep-alive)
    ///
    /// Comments are lines starting with `:` and are ignored by the client.
    /// They're useful for keeping the connection alive.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::sse::Event;
    ///
    /// let comment = Event::comment("keep-alive");
    /// assert_eq!(comment.format(), ": keep-alive\n\n");
    /// ```
    pub fn comment(text: impl Into<String>) -> String {
        format!(": {}\n\n", text.into())
    }
}

impl From<String> for Event {
    fn from(data: String) -> Self {
        Self::new(data)
    }
}

impl From<&str> for Event {
    fn from(data: &str) -> Self {
        Self::new(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let event = Event::new("test data");
        assert_eq!(event.data, "test data");
        assert_eq!(event.event, None);
        assert_eq!(event.id, None);
        assert_eq!(event.retry, None);
    }

    #[test]
    fn test_event_data() {
        let event = Event::data("test");
        assert_eq!(event.data, "test");
    }

    #[test]
    fn test_event_with_event() {
        let event = Event::new("data").with_event("custom");
        assert_eq!(event.event, Some("custom".to_string()));
    }

    #[test]
    fn test_event_with_id() {
        let event = Event::new("data").with_id("123");
        assert_eq!(event.id, Some("123".to_string()));
    }

    #[test]
    fn test_event_with_retry() {
        let event = Event::new("data").with_retry(5000);
        assert_eq!(event.retry, Some(5000));
    }

    #[test]
    fn test_event_format_simple() {
        let event = Event::new("Hello, world!");
        assert_eq!(event.format(), "data: Hello, world!\n\n");
    }

    #[test]
    fn test_event_format_with_event_type() {
        let event = Event::new("User joined").with_event("user-join");
        assert_eq!(event.format(), "event: user-join\ndata: User joined\n\n");
    }

    #[test]
    fn test_event_format_with_id() {
        let event = Event::new("Data").with_id("123");
        assert_eq!(event.format(), "data: Data\nid: 123\n\n");
    }

    #[test]
    fn test_event_format_with_retry() {
        let event = Event::new("Data").with_retry(3000);
        assert_eq!(event.format(), "data: Data\nretry: 3000\n\n");
    }

    #[test]
    fn test_event_format_complete() {
        let event = Event::new("Complete event")
            .with_event("test")
            .with_id("42")
            .with_retry(5000);

        assert_eq!(
            event.format(),
            "event: test\ndata: Complete event\nid: 42\nretry: 5000\n\n"
        );
    }

    #[test]
    fn test_event_format_multiline_data() {
        let event = Event::new("Line 1\nLine 2\nLine 3");
        assert_eq!(
            event.format(),
            "data: Line 1\ndata: Line 2\ndata: Line 3\n\n"
        );
    }

    #[test]
    fn test_event_format_empty_data() {
        let event = Event::new("");
        assert_eq!(event.format(), "data: \n\n");
    }

    #[test]
    fn test_event_comment() {
        let comment = Event::comment("keep-alive");
        assert_eq!(comment, ": keep-alive\n\n");
    }

    #[test]
    fn test_event_from_string() {
        let event: Event = "test data".into();
        assert_eq!(event.data, "test data");
    }

    #[test]
    fn test_event_from_str() {
        let event: Event = String::from("test data").into();
        assert_eq!(event.data, "test data");
    }

    #[test]
    fn test_event_clone() {
        let event1 = Event::new("data").with_event("test").with_id("1");
        let event2 = event1.clone();
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_event_equality() {
        let event1 = Event::new("data").with_event("test");
        let event2 = Event::new("data").with_event("test");
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_event_builder_pattern() {
        let event = Event::new("User logged in")
            .with_event("auth")
            .with_id("user-123")
            .with_retry(10000);

        assert_eq!(event.event, Some("auth".to_string()));
        assert_eq!(event.data, "User logged in");
        assert_eq!(event.id, Some("user-123".to_string()));
        assert_eq!(event.retry, Some(10000));
    }

    #[test]
    fn test_event_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Event>();
        assert_sync::<Event>();
    }
}
