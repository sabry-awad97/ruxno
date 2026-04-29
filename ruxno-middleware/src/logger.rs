//! Request/response logging middleware
//!
//! Logs HTTP requests and responses with timing information.

/// Logger middleware (placeholder)
///
/// TODO: Implement request/response logging
pub struct LoggerMiddleware;

impl LoggerMiddleware {
    /// Create a new logger middleware
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
