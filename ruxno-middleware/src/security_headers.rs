//! Security headers middleware
//!
//! Adds security-related HTTP headers (HSTS, CSP, X-Frame-Options, etc.).

/// Security headers middleware (placeholder)
///
/// TODO: Implement security headers
pub struct SecurityHeadersMiddleware;

impl SecurityHeadersMiddleware {
    /// Create a new security headers middleware
    pub fn new() -> Self {
        Self
    }
}

impl Default for SecurityHeadersMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
