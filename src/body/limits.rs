//! Body size limits

/// Body size limits
pub struct BodyLimits {
    /// Maximum body size in bytes
    max_size: usize,
}

impl BodyLimits {
    /// Create new limits
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Get maximum size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Check if size is within limits
    pub fn check(&self, size: usize) -> bool {
        size <= self.max_size
    }
}

impl Default for BodyLimits {
    fn default() -> Self {
        Self::new(1024 * 1024) // 1MB default
    }
}
