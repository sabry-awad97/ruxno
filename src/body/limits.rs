//! Body size limits and validation
//!
//! This module provides utilities for validating request body sizes
//! and returning appropriate HTTP 413 (Payload Too Large) responses.
//!
//! # Design
//!
//! - **Configurable limits**: Set max body size per route or globally
//! - **HTTP 413**: Returns proper status code on overflow
//! - **Early validation**: Check size before parsing
//! - **Flexible**: Can be used as utility or middleware
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::BodyLimits;
//!
//! let limits = BodyLimits::new(1024 * 1024); // 1MB
//! if !limits.check(body_size) {
//!     // Return 413 error
//! }
//! ```

use crate::core::{CoreError, StatusCode};
use bytes::Bytes;

/// Default maximum body size (2MB)
const DEFAULT_MAX_SIZE: usize = 2 * 1024 * 1024;

/// Body size limits configuration
///
/// Provides utilities for validating request body sizes and
/// generating appropriate error responses.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::BodyLimits;
///
/// // Create with default limit (2MB)
/// let limits = BodyLimits::default();
///
/// // Create with custom limit
/// let limits = BodyLimits::new(5 * 1024 * 1024); // 5MB
///
/// // Check size
/// if !limits.check(body.len()) {
///     return Err(limits.error(body.len()));
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyLimits {
    /// Maximum body size in bytes
    max_size: usize,
}

impl BodyLimits {
    /// Create new body limits with specified max size
    ///
    /// # Arguments
    ///
    /// - `max_size`: Maximum body size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024 * 1024); // 1MB
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Create limits for small bodies (256KB)
    ///
    /// Useful for API endpoints that expect small JSON payloads.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::small(); // 256KB
    /// ```
    pub fn small() -> Self {
        Self::new(256 * 1024)
    }

    /// Create limits for medium bodies (2MB) - default
    ///
    /// Suitable for most API use cases.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::medium(); // 2MB
    /// ```
    pub fn medium() -> Self {
        Self::new(DEFAULT_MAX_SIZE)
    }

    /// Create limits for large bodies (10MB)
    ///
    /// Useful for file upload endpoints.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::large(); // 10MB
    /// ```
    pub fn large() -> Self {
        Self::new(10 * 1024 * 1024)
    }

    /// Create limits for very large bodies (50MB)
    ///
    /// Use with caution - may impact memory usage.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::very_large(); // 50MB
    /// ```
    pub fn very_large() -> Self {
        Self::new(50 * 1024 * 1024)
    }

    /// Create unlimited body size (use with extreme caution)
    ///
    /// This effectively disables size checking. Only use when
    /// you have other mechanisms to prevent memory exhaustion.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::unlimited(); // usize::MAX
    /// ```
    pub fn unlimited() -> Self {
        Self::new(usize::MAX)
    }

    /// Get the configured maximum size
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024);
    /// assert_eq!(limits.max_size(), 1024);
    /// ```
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Check if a size is within limits
    ///
    /// Returns `true` if the size is acceptable, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// - `size`: Size to check in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024);
    /// assert!(limits.check(512));
    /// assert!(!limits.check(2048));
    /// ```
    pub fn check(&self, size: usize) -> bool {
        size <= self.max_size
    }

    /// Validate body size and return error if exceeded
    ///
    /// This is a convenience method that checks the size and
    /// returns an appropriate error if it exceeds the limit.
    ///
    /// # Arguments
    ///
    /// - `size`: Size to validate in bytes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if within limits, or an error with HTTP 413 status.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024);
    /// limits.validate(512)?; // OK
    /// limits.validate(2048)?; // Error
    /// ```
    pub fn validate(&self, size: usize) -> Result<(), CoreError> {
        if self.check(size) {
            Ok(())
        } else {
            Err(self.error(size))
        }
    }

    /// Validate bytes and return error if exceeded
    ///
    /// Convenience method for validating `Bytes` objects.
    ///
    /// # Arguments
    ///
    /// - `bytes`: Bytes to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if within limits, or an error with HTTP 413 status.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024);
    /// limits.validate_bytes(&body)?;
    /// ```
    pub fn validate_bytes(&self, bytes: &Bytes) -> Result<(), CoreError> {
        self.validate(bytes.len())
    }

    /// Create an error for size limit exceeded
    ///
    /// Generates a `CoreError` with HTTP 413 status and a
    /// descriptive message.
    ///
    /// # Arguments
    ///
    /// - `actual_size`: The actual size that exceeded the limit
    ///
    /// # Returns
    ///
    /// Returns a `CoreError` with status 413 (Payload Too Large).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024);
    /// let error = limits.error(2048);
    /// ```
    pub fn error(&self, actual_size: usize) -> CoreError {
        CoreError::payload_too_large(format!(
            "Request body too large: {} bytes (max: {} bytes)",
            actual_size, self.max_size
        ))
    }

    /// Format size in human-readable format
    ///
    /// Converts bytes to KB, MB, or GB as appropriate.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(BodyLimits::format_size(1024), "1.00 KB");
    /// assert_eq!(BodyLimits::format_size(1048576), "1.00 MB");
    /// ```
    pub fn format_size(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }

    /// Get human-readable description of limits
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limits = BodyLimits::new(1024 * 1024);
    /// println!("{}", limits.description()); // "1.00 MB"
    /// ```
    pub fn description(&self) -> String {
        Self::format_size(self.max_size)
    }
}

impl Default for BodyLimits {
    fn default() -> Self {
        Self::medium()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_limits_new() {
        let limits = BodyLimits::new(1024);
        assert_eq!(limits.max_size(), 1024);
    }

    #[test]
    fn test_body_limits_small() {
        let limits = BodyLimits::small();
        assert_eq!(limits.max_size(), 256 * 1024);
    }

    #[test]
    fn test_body_limits_medium() {
        let limits = BodyLimits::medium();
        assert_eq!(limits.max_size(), 2 * 1024 * 1024);
    }

    #[test]
    fn test_body_limits_large() {
        let limits = BodyLimits::large();
        assert_eq!(limits.max_size(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_body_limits_very_large() {
        let limits = BodyLimits::very_large();
        assert_eq!(limits.max_size(), 50 * 1024 * 1024);
    }

    #[test]
    fn test_body_limits_unlimited() {
        let limits = BodyLimits::unlimited();
        assert_eq!(limits.max_size(), usize::MAX);
    }

    #[test]
    fn test_body_limits_default() {
        let limits = BodyLimits::default();
        assert_eq!(limits.max_size(), DEFAULT_MAX_SIZE);
    }

    #[test]
    fn test_check_within_limit() {
        let limits = BodyLimits::new(1024);
        assert!(limits.check(512));
        assert!(limits.check(1024));
    }

    #[test]
    fn test_check_exceeds_limit() {
        let limits = BodyLimits::new(1024);
        assert!(!limits.check(1025));
        assert!(!limits.check(2048));
    }

    #[test]
    fn test_check_zero_size() {
        let limits = BodyLimits::new(1024);
        assert!(limits.check(0));
    }

    #[test]
    fn test_validate_success() {
        let limits = BodyLimits::new(1024);
        assert!(limits.validate(512).is_ok());
        assert!(limits.validate(1024).is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let limits = BodyLimits::new(1024);
        let result = limits.validate(2048);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn test_validate_bytes_success() {
        let limits = BodyLimits::new(1024);
        let bytes = Bytes::from(vec![0u8; 512]);
        assert!(limits.validate_bytes(&bytes).is_ok());
    }

    #[test]
    fn test_validate_bytes_failure() {
        let limits = BodyLimits::new(1024);
        let bytes = Bytes::from(vec![0u8; 2048]);
        let result = limits.validate_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_message() {
        let limits = BodyLimits::new(1024);
        let error = limits.error(2048);

        assert_eq!(error.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
        assert!(error.to_string().contains("2048"));
        assert!(error.to_string().contains("1024"));
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(BodyLimits::format_size(512), "512 bytes");
        assert_eq!(BodyLimits::format_size(1023), "1023 bytes");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(BodyLimits::format_size(1024), "1.00 KB");
        assert_eq!(BodyLimits::format_size(2048), "2.00 KB");
        assert_eq!(BodyLimits::format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(BodyLimits::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(BodyLimits::format_size(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(BodyLimits::format_size(1536 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(BodyLimits::format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(BodyLimits::format_size(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn test_description() {
        let limits = BodyLimits::new(1024 * 1024);
        assert_eq!(limits.description(), "1.00 MB");
    }

    #[test]
    fn test_limits_is_copy() {
        let limits = BodyLimits::new(1024);
        let copied = limits;
        assert_eq!(limits.max_size(), copied.max_size());
    }

    #[test]
    fn test_limits_is_clone() {
        let limits = BodyLimits::new(1024);
        let cloned = limits.clone();
        assert_eq!(limits.max_size(), cloned.max_size());
    }

    #[test]
    fn test_limits_equality() {
        let limits1 = BodyLimits::new(1024);
        let limits2 = BodyLimits::new(1024);
        let limits3 = BodyLimits::new(2048);

        assert_eq!(limits1, limits2);
        assert_ne!(limits1, limits3);
    }
}
