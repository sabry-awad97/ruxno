//! Core error types
//!
//! This module defines the core error types used throughout the framework.
//! All errors implement `std::error::Error` via the `thiserror` crate for
//! ergonomic error handling.
//!
//! # Error Categories
//!
//! - **Routing Errors**: Route not found, method not allowed, pattern errors
//! - **Request Errors**: Bad request, invalid parameters, body parsing errors
//! - **Internal Errors**: Framework bugs, unexpected states
//!
//! # Examples
//!
//! ```rust
//! use ruxno::CoreError;
//!
//! // Create routing errors
//! let error = CoreError::not_found("/api/users");
//! assert_eq!(error.status_code(), 404);
//!
//! // Create validation errors
//! let error = CoreError::bad_request("Invalid email format");
//! assert_eq!(error.status_code(), 400);
//!
//! // Pattern matching
//! match error {
//!     CoreError::NotFound(path) => println!("Path not found: {}", path),
//!     CoreError::BadRequest(msg) => println!("Bad request: {}", msg),
//!     _ => println!("Other error"),
//! }
//! ```

use thiserror::Error;

/// Core error type for the framework
///
/// This enum represents all errors that can occur in the core framework layers
/// (routing, middleware, handlers). Each variant maps to an appropriate HTTP
/// status code.
///
/// # Design
///
/// Uses `thiserror` for automatic `std::error::Error` implementation and
/// user-friendly error messages. All variants include context to aid debugging.
///
/// # Examples
///
/// ```rust
/// use ruxno::CoreError;
///
/// // Routing errors
/// let error = CoreError::not_found("/api/users");
/// assert_eq!(error.status_code(), 404);
///
/// // Request errors
/// let error = CoreError::bad_request("Missing required field: email");
/// assert_eq!(error.status_code(), 400);
///
/// // Pattern errors
/// let error = CoreError::invalid_pattern("Invalid route pattern: /user/:id:");
/// assert_eq!(error.status_code(), 500);
/// ```
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CoreError {
    /// Route not found (404)
    ///
    /// Returned when no route matches the requested path and method.
    #[error("Route not found: {0}")]
    NotFound(String),

    /// Method not allowed (405)
    ///
    /// Returned when a route exists for the path but not for the HTTP method.
    #[error("Method not allowed for path: {0}")]
    MethodNotAllowed(String),

    /// Bad request (400)
    ///
    /// Returned for malformed requests, invalid parameters, or validation errors.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Invalid route pattern (500)
    ///
    /// Returned when registering a route with an invalid pattern.
    /// This is a framework error, not a client error.
    #[error("Invalid route pattern: {0}")]
    InvalidPattern(String),

    /// Duplicate route (500)
    ///
    /// Returned when attempting to register a route that already exists.
    /// This is a framework error, not a client error.
    #[error("Duplicate route: {method} {path}")]
    DuplicateRoute {
        /// HTTP method
        method: String,
        /// Route path
        path: String,
    },

    /// Missing parameter (400)
    ///
    /// Returned when a required path parameter is missing.
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// Invalid parameter (400)
    ///
    /// Returned when a path parameter has an invalid value.
    #[error("Invalid parameter '{name}': {reason}")]
    InvalidParameter {
        /// Parameter name
        name: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Body parsing error (400)
    ///
    /// Returned when request body cannot be parsed (JSON, form, etc.).
    #[error("Failed to parse request body: {0}")]
    BodyParseError(String),

    /// Payload too large (413)
    ///
    /// Returned when request body exceeds size limits.
    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    /// Internal error (500)
    ///
    /// Returned for unexpected framework errors or bugs.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Custom error (500)
    ///
    /// Allows users to create custom errors with specific messages.
    #[error("{0}")]
    Custom(String),
}

impl CoreError {
    /// Get the HTTP status code for this error
    ///
    /// Maps each error variant to its appropriate HTTP status code:
    /// - 400: Bad request, validation errors
    /// - 404: Route not found
    /// - 405: Method not allowed
    /// - 500: Internal errors, framework bugs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ruxno::CoreError;
    ///
    /// assert_eq!(CoreError::not_found("/api/users").status_code(), 404);
    /// assert_eq!(CoreError::bad_request("Invalid input").status_code(), 400);
    /// assert_eq!(CoreError::internal("Bug").status_code(), 500);
    /// ```
    pub fn status_code(&self) -> u16 {
        match self {
            CoreError::NotFound(_) => 404,
            CoreError::MethodNotAllowed(_) => 405,
            CoreError::BadRequest(_) => 400,
            CoreError::MissingParameter(_) => 400,
            CoreError::InvalidParameter { .. } => 400,
            CoreError::BodyParseError(_) => 400,
            CoreError::PayloadTooLarge(_) => 413,
            CoreError::InvalidPattern(_) => 500,
            CoreError::DuplicateRoute { .. } => 500,
            CoreError::Internal(_) => 500,
            CoreError::Custom(_) => 500,
        }
    }

    /// Check if this is a client error (4xx)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ruxno::CoreError;
    ///
    /// assert!(CoreError::not_found("/api/users").is_client_error());
    /// assert!(CoreError::bad_request("Invalid").is_client_error());
    /// assert!(!CoreError::internal("Bug").is_client_error());
    /// ```
    pub fn is_client_error(&self) -> bool {
        let code = self.status_code();
        (400..500).contains(&code)
    }

    /// Check if this is a server error (5xx)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ruxno::CoreError;
    ///
    /// assert!(CoreError::internal("Bug").is_server_error());
    /// assert!(CoreError::invalid_pattern("Bad pattern").is_server_error());
    /// assert!(!CoreError::not_found("/api/users").is_server_error());
    /// ```
    pub fn is_server_error(&self) -> bool {
        let code = self.status_code();
        (500..600).contains(&code)
    }

    // Convenience constructors

    /// Create a "not found" error
    pub fn not_found(path: impl Into<String>) -> Self {
        CoreError::NotFound(path.into())
    }

    /// Create a "method not allowed" error
    pub fn method_not_allowed(path: impl Into<String>) -> Self {
        CoreError::MethodNotAllowed(path.into())
    }

    /// Create a "bad request" error
    pub fn bad_request(message: impl Into<String>) -> Self {
        CoreError::BadRequest(message.into())
    }

    /// Create an "invalid pattern" error
    pub fn invalid_pattern(pattern: impl Into<String>) -> Self {
        CoreError::InvalidPattern(pattern.into())
    }

    /// Create a "duplicate route" error
    pub fn duplicate_route(method: impl Into<String>, path: impl Into<String>) -> Self {
        CoreError::DuplicateRoute {
            method: method.into(),
            path: path.into(),
        }
    }

    /// Create a "missing parameter" error
    pub fn missing_parameter(name: impl Into<String>) -> Self {
        CoreError::MissingParameter(name.into())
    }

    /// Create an "invalid parameter" error
    pub fn invalid_parameter(name: impl Into<String>, reason: impl Into<String>) -> Self {
        CoreError::InvalidParameter {
            name: name.into(),
            reason: reason.into(),
        }
    }

    /// Create a "body parse error"
    pub fn body_parse_error(message: impl Into<String>) -> Self {
        CoreError::BodyParseError(message.into())
    }

    /// Create a "payload too large" error
    pub fn payload_too_large(message: impl Into<String>) -> Self {
        CoreError::PayloadTooLarge(message.into())
    }

    /// Create an "internal error"
    pub fn internal(message: impl Into<String>) -> Self {
        CoreError::Internal(message.into())
    }

    /// Create a "custom error"
    pub fn custom(message: impl Into<String>) -> Self {
        CoreError::Custom(message.into())
    }
}

// Conversion from common error types

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        CoreError::Internal(err.to_string())
    }
}

impl From<std::fmt::Error> for CoreError {
    fn from(err: std::fmt::Error) -> Self {
        CoreError::Internal(err.to_string())
    }
}

impl From<String> for CoreError {
    fn from(msg: String) -> Self {
        CoreError::Custom(msg)
    }
}

impl From<&str> for CoreError {
    fn from(msg: &str) -> Self {
        CoreError::Custom(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let error = CoreError::not_found("/api/users");
        assert_eq!(error.status_code(), 404);
        assert!(error.is_client_error());
        assert!(!error.is_server_error());
        assert_eq!(error.to_string(), "Route not found: /api/users");
    }

    #[test]
    fn test_method_not_allowed_error() {
        let error = CoreError::method_not_allowed("/api/users");
        assert_eq!(error.status_code(), 405);
        assert!(error.is_client_error());
        assert_eq!(error.to_string(), "Method not allowed for path: /api/users");
    }

    #[test]
    fn test_bad_request_error() {
        let error = CoreError::bad_request("Invalid email format");
        assert_eq!(error.status_code(), 400);
        assert!(error.is_client_error());
        assert_eq!(error.to_string(), "Bad request: Invalid email format");
    }

    #[test]
    fn test_invalid_pattern_error() {
        let error = CoreError::invalid_pattern("/user/:id:");
        assert_eq!(error.status_code(), 500);
        assert!(error.is_server_error());
        assert!(!error.is_client_error());
        assert_eq!(error.to_string(), "Invalid route pattern: /user/:id:");
    }

    #[test]
    fn test_duplicate_route_error() {
        let error = CoreError::duplicate_route("GET", "/api/users");
        assert_eq!(error.status_code(), 500);
        assert!(error.is_server_error());
        assert_eq!(error.to_string(), "Duplicate route: GET /api/users");
    }

    #[test]
    fn test_missing_parameter_error() {
        let error = CoreError::missing_parameter("id");
        assert_eq!(error.status_code(), 400);
        assert!(error.is_client_error());
        assert_eq!(error.to_string(), "Missing required parameter: id");
    }

    #[test]
    fn test_invalid_parameter_error() {
        let error = CoreError::invalid_parameter("id", "must be a positive integer");
        assert_eq!(error.status_code(), 400);
        assert!(error.is_client_error());
        assert_eq!(
            error.to_string(),
            "Invalid parameter 'id': must be a positive integer"
        );
    }

    #[test]
    fn test_body_parse_error() {
        let error = CoreError::body_parse_error("Invalid JSON");
        assert_eq!(error.status_code(), 400);
        assert!(error.is_client_error());
        assert_eq!(
            error.to_string(),
            "Failed to parse request body: Invalid JSON"
        );
    }

    #[test]
    fn test_internal_error() {
        let error = CoreError::internal("Unexpected state");
        assert_eq!(error.status_code(), 500);
        assert!(error.is_server_error());
        assert_eq!(error.to_string(), "Internal error: Unexpected state");
    }

    #[test]
    fn test_custom_error() {
        let error = CoreError::custom("Custom error message");
        assert_eq!(error.status_code(), 500);
        assert!(error.is_server_error());
        assert_eq!(error.to_string(), "Custom error message");
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: CoreError = io_error.into();
        assert_eq!(error.status_code(), 500);
        assert!(error.is_server_error());
    }

    #[test]
    fn test_from_string() {
        let error: CoreError = "Something went wrong".to_string().into();
        assert_eq!(error.status_code(), 500);
        assert_eq!(error.to_string(), "Something went wrong");
    }

    #[test]
    fn test_from_str() {
        let error: CoreError = "Something went wrong".into();
        assert_eq!(error.status_code(), 500);
        assert_eq!(error.to_string(), "Something went wrong");
    }

    #[test]
    fn test_error_equality() {
        let error1 = CoreError::not_found("/api/users");
        let error2 = CoreError::not_found("/api/users");
        let error3 = CoreError::not_found("/api/posts");

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_clone() {
        let error = CoreError::bad_request("Invalid input");
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_status_code_categories() {
        // Client errors (4xx)
        assert!(CoreError::not_found("/").is_client_error());
        assert!(CoreError::method_not_allowed("/").is_client_error());
        assert!(CoreError::bad_request("").is_client_error());
        assert!(CoreError::missing_parameter("id").is_client_error());

        // Server errors (5xx)
        assert!(CoreError::internal("").is_server_error());
        assert!(CoreError::invalid_pattern("").is_server_error());
        assert!(CoreError::duplicate_route("GET", "/").is_server_error());
        assert!(CoreError::custom("").is_server_error());
    }
}
