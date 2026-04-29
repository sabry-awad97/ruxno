//! Response compression middleware
//!
//! Supports gzip and brotli compression.

/// Compression middleware (placeholder)
///
/// TODO: Implement compression support
pub struct CompressionMiddleware;

impl CompressionMiddleware {
    /// Create a new compression middleware
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
