//! Body utilities

use bytes::Bytes;

/// Body wrapper
pub struct Body {
    bytes: Bytes,
}

impl Body {
    /// Create new body
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    /// Get bytes
    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }

    /// Create empty body
    pub fn empty() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }
}

impl From<Bytes> for Body {
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::new(Bytes::from(s))
    }
}
