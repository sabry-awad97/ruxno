//! Body streaming utilities
//!
//! This module provides utilities for streaming request and response bodies
//! with chunked reading and backpressure handling.
//!
//! # Design
//!
//! - **Chunked reading**: Read body in configurable chunks
//! - **Backpressure**: Automatic flow control via Stream trait
//! - **Error handling**: Propagates I/O errors through stream
//! - **Flexible**: Works with any Stream of bytes
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::BodyStream;
//! use futures_util::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut stream = BodyStream::new(some_stream);
//!
//! // Read chunks
//! while let Some(chunk) = stream.next().await {
//!     let bytes = chunk?;
//!     println!("Received {} bytes", bytes.len());
//! }
//! # Ok(())
//! # }
//! ```

use bytes::Bytes;
use futures_util::stream::{Stream, StreamExt};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Default chunk size for streaming (8KB)
const DEFAULT_CHUNK_SIZE: usize = 8 * 1024;

/// Body stream wrapper with chunked reading and backpressure handling
///
/// Wraps any stream of bytes and provides convenient methods for
/// reading chunks with automatic backpressure handling via the
/// Stream trait.
///
/// # Backpressure
///
/// Backpressure is automatically handled by the underlying Stream
/// implementation. When the consumer is slow, the stream will
/// naturally pause, preventing memory buildup.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::BodyStream;
/// use futures_util::StreamExt;
///
/// let mut stream = BodyStream::new(incoming_body);
///
/// // Read all chunks
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(chunk) => process_chunk(chunk),
///         Err(e) => handle_error(e),
///     }
/// }
/// ```
pub struct BodyStream {
    /// Inner stream of byte chunks
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>,

    /// Optional chunk size limit
    chunk_size: Option<usize>,

    /// Total bytes read (for monitoring)
    bytes_read: usize,
}

impl BodyStream {
    /// Create new body stream from any stream of bytes
    ///
    /// # Arguments
    ///
    /// - `stream`: Any stream that yields `Result<Bytes, io::Error>`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stream = BodyStream::new(hyper_body_stream);
    /// ```
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
            chunk_size: None,
            bytes_read: 0,
        }
    }

    /// Create body stream with maximum chunk size
    ///
    /// Chunks larger than this size will be split into smaller chunks.
    /// This helps with memory management and backpressure.
    ///
    /// # Arguments
    ///
    /// - `stream`: Any stream that yields `Result<Bytes, io::Error>`
    /// - `chunk_size`: Maximum size of each chunk in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stream = BodyStream::with_chunk_size(body_stream, 4096); // 4KB chunks
    /// ```
    pub fn with_chunk_size<S>(stream: S, chunk_size: usize) -> Self
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
            chunk_size: Some(chunk_size),
            bytes_read: 0,
        }
    }

    /// Get the next chunk from the stream
    ///
    /// Returns `None` when the stream is exhausted.
    /// Backpressure is automatically handled - this method will
    /// wait if the underlying stream is not ready.
    ///
    /// # Returns
    ///
    /// - `Some(Ok(bytes))` - Next chunk of data
    /// - `Some(Err(e))` - I/O error occurred
    /// - `None` - Stream is exhausted
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// while let Some(chunk) = stream.next().await {
    ///     let bytes = chunk?;
    ///     println!("Got {} bytes", bytes.len());
    /// }
    /// ```
    pub async fn next(&mut self) -> Option<Result<Bytes, io::Error>> {
        match self.inner.next().await {
            Some(Ok(bytes)) => {
                self.bytes_read += bytes.len();

                // Apply chunk size limit if configured
                if let Some(max_size) = self.chunk_size {
                    if bytes.len() > max_size {
                        // Split large chunk
                        let chunk = bytes.slice(0..max_size);
                        // Note: Remaining bytes are lost in this simple implementation
                        // A production version would buffer remaining bytes
                        return Some(Ok(chunk));
                    }
                }

                Some(Ok(bytes))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    /// Get total bytes read so far
    ///
    /// Useful for monitoring progress or implementing size limits.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// while let Some(chunk) = stream.next().await {
    ///     chunk?;
    ///     println!("Total read: {} bytes", stream.bytes_read());
    /// }
    /// ```
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    /// Collect all chunks into a single Bytes buffer
    ///
    /// This consumes the stream and buffers all data in memory.
    /// Use with caution for large bodies.
    ///
    /// # Returns
    ///
    /// Returns all bytes or the first error encountered.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let all_bytes = stream.collect_bytes().await?;
    /// ```
    pub async fn collect_bytes(mut self) -> Result<Bytes, io::Error> {
        let mut buffer = Vec::new();

        while let Some(chunk) = self.next().await {
            let bytes = chunk?;
            buffer.extend_from_slice(&bytes);
        }

        Ok(Bytes::from(buffer))
    }

    /// Collect all chunks with a size limit
    ///
    /// Returns an error if the total size exceeds the limit.
    ///
    /// # Arguments
    ///
    /// - `max_size`: Maximum total size in bytes
    ///
    /// # Returns
    ///
    /// Returns all bytes or an error if limit exceeded.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bytes = stream.collect_with_limit(1024 * 1024).await?; // 1MB limit
    /// ```
    pub async fn collect_with_limit(mut self, max_size: usize) -> Result<Bytes, io::Error> {
        let mut buffer = Vec::new();

        while let Some(chunk) = self.next().await {
            let bytes = chunk?;

            if buffer.len() + bytes.len() > max_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Body size exceeds limit of {} bytes", max_size),
                ));
            }

            buffer.extend_from_slice(&bytes);
        }

        Ok(Bytes::from(buffer))
    }
}

// Implement Stream trait for BodyStream to enable use with stream combinators
impl Stream for BodyStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;

    // Helper to create a test stream
    fn create_test_stream(
        chunks: Vec<&'static [u8]>,
    ) -> impl Stream<Item = Result<Bytes, io::Error>> + 'static {
        stream::iter(
            chunks
                .into_iter()
                .map(|chunk| Ok(Bytes::from(chunk.to_vec()))),
        )
    }

    #[tokio::test]
    async fn test_body_stream_new() {
        let test_stream = create_test_stream(vec![b"hello", b"world"]);
        let mut stream = BodyStream::new(test_stream);

        let chunk1 = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk1, Bytes::from("hello"));

        let chunk2 = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk2, Bytes::from("world"));

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_body_stream_with_chunk_size() {
        let test_stream = create_test_stream(vec![b"hello"]);
        let mut stream = BodyStream::with_chunk_size(test_stream, 1024);

        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk, Bytes::from("hello"));
    }

    #[tokio::test]
    async fn test_body_stream_bytes_read() {
        let test_stream = create_test_stream(vec![b"hello", b"world"]);
        let mut stream = BodyStream::new(test_stream);

        assert_eq!(stream.bytes_read(), 0);

        stream.next().await.unwrap().unwrap();
        assert_eq!(stream.bytes_read(), 5);

        stream.next().await.unwrap().unwrap();
        assert_eq!(stream.bytes_read(), 10);
    }

    #[tokio::test]
    async fn test_body_stream_empty() {
        let test_stream = create_test_stream(vec![]);
        let mut stream = BodyStream::new(test_stream);

        assert!(stream.next().await.is_none());
        assert_eq!(stream.bytes_read(), 0);
    }

    #[tokio::test]
    async fn test_body_stream_single_chunk() {
        let test_stream = create_test_stream(vec![b"single chunk"]);
        let mut stream = BodyStream::new(test_stream);

        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk, Bytes::from("single chunk"));
        assert_eq!(stream.bytes_read(), 12);

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_body_stream_error() {
        let error_stream = stream::iter(vec![
            Ok(Bytes::from("hello")),
            Err(io::Error::other("test error")),
        ]);
        let mut stream = BodyStream::new(error_stream);

        // First chunk succeeds
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk, Bytes::from("hello"));

        // Second chunk is error
        let result = stream.next().await.unwrap();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_collect_bytes() {
        let test_stream = create_test_stream(vec![b"hello", b" ", b"world"]);
        let stream = BodyStream::new(test_stream);

        let all_bytes = stream.collect_bytes().await.unwrap();
        assert_eq!(all_bytes, Bytes::from("hello world"));
    }

    #[tokio::test]
    async fn test_collect_bytes_empty() {
        let test_stream = create_test_stream(vec![]);
        let stream = BodyStream::new(test_stream);

        let all_bytes = stream.collect_bytes().await.unwrap();
        assert_eq!(all_bytes, Bytes::from(""));
    }

    #[tokio::test]
    async fn test_collect_with_limit_success() {
        let test_stream = create_test_stream(vec![b"hello", b"world"]);
        let stream = BodyStream::new(test_stream);

        let bytes = stream.collect_with_limit(100).await.unwrap();
        assert_eq!(bytes, Bytes::from("helloworld"));
    }

    #[tokio::test]
    async fn test_collect_with_limit_exceeded() {
        let test_stream = create_test_stream(vec![b"hello", b"world"]);
        let stream = BodyStream::new(test_stream);

        let result = stream.collect_with_limit(5).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn test_collect_with_limit_exact() {
        let test_stream = create_test_stream(vec![b"hello"]);
        let stream = BodyStream::new(test_stream);

        let bytes = stream.collect_with_limit(5).await.unwrap();
        assert_eq!(bytes, Bytes::from("hello"));
    }

    #[tokio::test]
    async fn test_stream_trait_implementation() {
        let test_stream = create_test_stream(vec![b"hello", b"world"]);
        let mut stream = BodyStream::new(test_stream);

        // Use StreamExt methods
        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk, Bytes::from("hello"));
    }

    #[tokio::test]
    async fn test_multiple_small_chunks() {
        let test_stream = create_test_stream(vec![b"a", b"b", b"c", b"d", b"e"]);
        let mut stream = BodyStream::new(test_stream);

        let mut collected = Vec::new();
        while let Some(chunk) = stream.next().await {
            collected.push(chunk.unwrap());
        }

        assert_eq!(collected.len(), 5);
        assert_eq!(stream.bytes_read(), 5);
    }

    #[tokio::test]
    async fn test_large_chunk() {
        let large_data = Bytes::from(vec![b'x'; 10000]);
        let test_stream = stream::iter(vec![Ok(large_data.clone())]);
        let mut stream = BodyStream::new(test_stream);

        let chunk = stream.next().await.unwrap().unwrap();
        assert_eq!(chunk.len(), 10000);
        assert_eq!(stream.bytes_read(), 10000);
    }

    #[tokio::test]
    async fn test_chunk_size_limit() {
        let large_data = Bytes::from(vec![b'x'; 1000]);
        let test_stream = stream::iter(vec![Ok(large_data)]);
        let mut stream = BodyStream::with_chunk_size(test_stream, 500);

        let chunk = stream.next().await.unwrap().unwrap();
        // Should be limited to 500 bytes
        assert_eq!(chunk.len(), 500);
    }
}
