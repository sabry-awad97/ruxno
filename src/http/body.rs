//! HTTP body utilities
//!
//! This module provides utilities for working with HTTP request and response bodies.
//! It wraps Hyper's body types and provides convenient methods for buffering and streaming.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::http::Body;
//!
//! // Buffer entire body into memory
//! let bytes = body.to_bytes().await?;
//!
//! // Stream body chunks
//! let stream = body.into_stream();
//! ```

use bytes::Bytes;
use futures_util::stream::Stream;
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use std::pin::Pin;
use std::task::{Context, Poll};

/// HTTP body wrapper
///
/// Wraps Hyper's `Incoming` body type and provides convenient methods
/// for buffering and streaming body data.
///
/// # Examples
///
/// ```rust,ignore
/// // Buffer entire body
/// let body = Body::from(incoming);
/// let bytes = body.to_bytes().await?;
///
/// // Stream body chunks
/// let stream = body.into_stream();
/// while let Some(chunk) = stream.next().await {
///     // Process chunk
/// }
/// ```
pub struct Body {
    inner: BodyInner,
}

/// Internal body representation
enum BodyInner {
    /// Hyper incoming body (streaming)
    Incoming(Incoming),
    /// Buffered bytes
    Bytes(Bytes),
    /// Generic stream
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>),
    /// Empty body
    Empty,
}

impl Body {
    /// Create body from a stream
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stream = futures_util::stream::iter(vec![
    ///     Ok(Bytes::from("Hello")),
    ///     Ok(Bytes::from(" World")),
    /// ]);
    /// let body = Body::from_stream(stream);
    /// ```
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
    {
        Self {
            inner: BodyInner::Stream(Box::pin(stream)),
        }
    }

    /// Create empty body
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = Body::empty();
    /// assert!(body.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            inner: BodyInner::Empty,
        }
    }

    /// Check if body is empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = Body::empty();
    /// assert!(body.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            BodyInner::Empty => true,
            BodyInner::Bytes(b) => b.is_empty(),
            BodyInner::Incoming(_) => false, // Can't know without consuming
            BodyInner::Stream(_) => false,   // Can't know without consuming
        }
    }

    /// Buffer entire body into memory
    ///
    /// This method consumes the body and returns all data as a single `Bytes` buffer.
    /// For streaming bodies, this will read all chunks into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the body fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = Body::from_incoming(incoming);
    /// let bytes = body.to_bytes().await?;
    /// println!("Body size: {} bytes", bytes.len());
    /// ```
    pub async fn to_bytes(self) -> Result<Bytes, BodyError> {
        use futures_util::StreamExt;

        match self.inner {
            BodyInner::Incoming(incoming) => {
                // Buffer the streaming body
                incoming
                    .collect()
                    .await
                    .map(|collected| collected.to_bytes())
                    .map_err(|e| BodyError::ReadError(e.to_string()))
            }
            BodyInner::Bytes(bytes) => Ok(bytes),
            BodyInner::Stream(mut stream) => {
                // Collect all chunks from the stream
                let mut buffer = Vec::new();
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => buffer.extend_from_slice(&chunk),
                        Err(e) => return Err(BodyError::ReadError(e.to_string())),
                    }
                }
                Ok(Bytes::from(buffer))
            }
            BodyInner::Empty => Ok(Bytes::new()),
        }
    }

    /// Convert body into a stream of chunks
    ///
    /// This method consumes the body and returns a stream that yields
    /// chunks of data as they become available.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let body = Body::from_incoming(incoming);
    /// let mut stream = body.into_stream();
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(chunk) => println!("Received {} bytes", chunk.len()),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub fn into_stream(self) -> BodyStream {
        match self.inner {
            BodyInner::Incoming(incoming) => BodyStream::Incoming(incoming),
            BodyInner::Bytes(bytes) => BodyStream::Buffered(Some(bytes)),
            BodyInner::Stream(stream) => BodyStream::Generic(stream),
            BodyInner::Empty => BodyStream::Buffered(None),
        }
    }
}

/// Body stream
///
/// A stream of body chunks. Can be created from either a streaming
/// Hyper body or buffered bytes.
pub enum BodyStream {
    /// Streaming from Hyper incoming body
    Incoming(Incoming),
    /// Buffered bytes (yields once then ends)
    Buffered(Option<Bytes>),
    /// Generic stream
    Generic(Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>),
}

impl Stream for BodyStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut *self {
            BodyStream::Incoming(incoming) => {
                // Poll the incoming body for the next frame
                match Pin::new(incoming).poll_frame(cx) {
                    Poll::Ready(Some(Ok(frame))) => {
                        // Extract data from frame if available
                        match frame.into_data() {
                            Ok(data) => Poll::Ready(Some(Ok(data))),
                            Err(_) => {
                                // Frame was not data (e.g., trailers), poll again
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                        }
                    }
                    Poll::Ready(Some(Err(e))) => {
                        Poll::Ready(Some(Err(std::io::Error::other(e.to_string()))))
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            BodyStream::Buffered(bytes) => {
                // Yield buffered bytes once, then end stream
                Poll::Ready(bytes.take().map(Ok))
            }
            BodyStream::Generic(stream) => {
                // Poll the generic stream
                match stream.as_mut().poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(bytes))),
                    Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

/// Body error
///
/// Errors that can occur when reading body data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyError {
    /// Error reading body data
    ReadError(String),
}

impl std::fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyError::ReadError(msg) => write!(f, "Body read error: {}", msg),
        }
    }
}

impl std::error::Error for BodyError {}

// Conversion traits

impl From<Incoming> for Body {
    /// Create body from Hyper incoming body
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = Body::from(incoming);
    /// ```
    fn from(incoming: Incoming) -> Self {
        Self {
            inner: BodyInner::Incoming(incoming),
        }
    }
}

impl From<Bytes> for Body {
    /// Create body from bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = Body::from_bytes(Bytes::from("Hello, World!"));
    /// ```
    fn from(bytes: Bytes) -> Self {
        Self {
            inner: BodyInner::Bytes(bytes),
        }
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self {
            inner: BodyInner::Bytes(Bytes::from(s)),
        }
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Self {
            inner: BodyInner::Bytes(Bytes::from(s.to_string())),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Self {
            inner: BodyInner::Bytes(Bytes::from(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_empty() {
        let body = Body::empty();
        assert!(body.is_empty());
    }

    #[test]
    fn test_body_from_bytes() {
        let bytes = Bytes::from("Hello, World!");
        let body = Body::from(bytes.clone());
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn test_body_to_bytes_from_bytes() {
        let bytes = Bytes::from("Hello, World!");
        let body = Body::from(bytes.clone());
        let result = body.to_bytes().await.unwrap();
        assert_eq!(result, bytes);
    }

    #[tokio::test]
    async fn test_body_to_bytes_empty() {
        let body = Body::empty();
        let result = body.to_bytes().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_body_stream_buffered() {
        use futures_util::StreamExt;

        let bytes = Bytes::from("Hello, World!");
        let body = Body::from(bytes.clone());
        let mut stream = body.into_stream();

        // First chunk should be the bytes
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk, bytes);

        // Stream should end
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_body_stream_empty() {
        use futures_util::StreamExt;

        let body = Body::empty();
        let mut stream = body.into_stream();

        // Stream should end immediately
        assert!(stream.next().await.is_none());
    }

    #[test]
    fn test_body_from_string() {
        let body = Body::from("Hello, World!".to_string());
        assert!(!body.is_empty());
    }

    #[test]
    fn test_body_from_str() {
        let body = Body::from("Hello, World!");
        assert!(!body.is_empty());
    }

    #[test]
    fn test_body_from_vec() {
        let body = Body::from(vec![1, 2, 3, 4, 5]);
        assert!(!body.is_empty());
    }

    #[test]
    fn test_body_error_display() {
        let error = BodyError::ReadError("test error".to_string());
        assert_eq!(error.to_string(), "Body read error: test error");
    }

    #[test]
    fn test_body_error_equality() {
        let error1 = BodyError::ReadError("test".to_string());
        let error2 = BodyError::ReadError("test".to_string());
        let error3 = BodyError::ReadError("other".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[tokio::test]
    async fn test_body_to_bytes_from_string() {
        let body = Body::from("Hello, World!");
        let result = body.to_bytes().await.unwrap();
        assert_eq!(result, Bytes::from("Hello, World!"));
    }

    #[tokio::test]
    async fn test_body_to_bytes_from_vec() {
        let body = Body::from(vec![1, 2, 3, 4, 5]);
        let result = body.to_bytes().await.unwrap();
        assert_eq!(result, Bytes::from(vec![1, 2, 3, 4, 5]));
    }
}
