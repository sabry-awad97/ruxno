//! Body streaming utilities

use bytes::Bytes;
use futures_util::Stream;
use std::pin::Pin;

/// Body stream
pub struct BodyStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
}

impl BodyStream {
    /// Create new body stream
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Get next chunk
    pub async fn next(&mut self) -> Option<Result<Bytes, std::io::Error>> {
        // TODO: Implement stream next
        todo!("Implement BodyStream::next")
    }
}
