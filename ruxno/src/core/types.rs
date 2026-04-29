//! Core types - Type aliases for HTTP primitives
//!
//! This module provides type aliases to the `http` crate's types, creating
//! a stable API boundary. If we ever need to switch HTTP implementations,
//! we only need to update these aliases.
//!
//! # Design Decision
//!
//! We use type aliases instead of custom enums to:
//! - Avoid unnecessary conversions and allocations
//! - Leverage the battle-tested `http` crate types
//! - Maintain zero-cost abstractions
//! - Provide a stable public API that can be changed internally
//!
//! # Examples
//!
//! ```rust
//! use ruxno::{Method, StatusCode};
//!
//! // Use exactly like http crate types
//! let method = Method::GET;
//! let status = StatusCode::OK;
//!
//! // Pattern matching works
//! match method {
//!     Method::GET => println!("GET request"),
//!     Method::POST => println!("POST request"),
//!     _ => println!("Other method"),
//! }
//! ```

/// HTTP request method
///
/// Type alias for `http::Method`. Supports all standard HTTP methods:
/// GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT, TRACE.
///
/// # Examples
///
/// ```rust
/// use ruxno::Method;
///
/// let method = Method::GET;
/// assert_eq!(method, Method::GET);
///
/// // Convert from string
/// let method: Method = "POST".parse().unwrap();
/// assert_eq!(method, Method::POST);
/// ```
pub type Method = http::Method;

/// HTTP status code
///
/// Type alias for `http::StatusCode`. Provides constants for all standard
/// HTTP status codes (100-599).
///
/// # Examples
///
/// ```rust
/// use ruxno::StatusCode;
///
/// let status = StatusCode::OK;
/// assert_eq!(status.as_u16(), 200);
///
/// let status = StatusCode::NOT_FOUND;
/// assert_eq!(status.as_u16(), 404);
///
/// // Create from u16
/// let status = StatusCode::from_u16(201).unwrap();
/// assert_eq!(status, StatusCode::CREATED);
/// ```
pub type StatusCode = http::StatusCode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_type_alias() {
        // Verify Method is usable
        let method = Method::GET;
        assert_eq!(method, Method::GET);
        assert_ne!(method, Method::POST);

        // Verify all standard methods are available
        let _get = Method::GET;
        let _post = Method::POST;
        let _put = Method::PUT;
        let _delete = Method::DELETE;
        let _patch = Method::PATCH;
        let _head = Method::HEAD;
        let _options = Method::OPTIONS;
    }

    #[test]
    fn test_method_parsing() {
        // Test parsing from string
        let method: Method = "GET".parse().unwrap();
        assert_eq!(method, Method::GET);

        let method: Method = "POST".parse().unwrap();
        assert_eq!(method, Method::POST);
    }

    #[test]
    fn test_method_as_str() {
        // Test conversion to string
        assert_eq!(Method::GET.as_str(), "GET");
        assert_eq!(Method::POST.as_str(), "POST");
        assert_eq!(Method::PUT.as_str(), "PUT");
        assert_eq!(Method::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_status_code_type_alias() {
        // Verify StatusCode is usable
        let status = StatusCode::OK;
        assert_eq!(status.as_u16(), 200);

        // Verify common status codes
        assert_eq!(StatusCode::OK.as_u16(), 200);
        assert_eq!(StatusCode::CREATED.as_u16(), 201);
        assert_eq!(StatusCode::NO_CONTENT.as_u16(), 204);
        assert_eq!(StatusCode::BAD_REQUEST.as_u16(), 400);
        assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);
    }

    #[test]
    fn test_status_code_from_u16() {
        // Test creating from u16
        let status = StatusCode::from_u16(200).unwrap();
        assert_eq!(status, StatusCode::OK);

        let status = StatusCode::from_u16(404).unwrap();
        assert_eq!(status, StatusCode::NOT_FOUND);

        // Test invalid status code (out of valid range 100-999)
        assert!(StatusCode::from_u16(99).is_err());
        assert!(StatusCode::from_u16(1000).is_err());
    }

    #[test]
    fn test_status_code_is_success() {
        // Test status code categories
        assert!(StatusCode::OK.is_success());
        assert!(StatusCode::CREATED.is_success());
        assert!(!StatusCode::BAD_REQUEST.is_success());
        assert!(!StatusCode::NOT_FOUND.is_success());
    }

    #[test]
    fn test_status_code_is_client_error() {
        assert!(!StatusCode::OK.is_client_error());
        assert!(StatusCode::BAD_REQUEST.is_client_error());
        assert!(StatusCode::NOT_FOUND.is_client_error());
        assert!(!StatusCode::INTERNAL_SERVER_ERROR.is_client_error());
    }

    #[test]
    fn test_status_code_is_server_error() {
        assert!(!StatusCode::OK.is_server_error());
        assert!(!StatusCode::BAD_REQUEST.is_server_error());
        assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error());
    }
}
