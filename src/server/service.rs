//! Hyper service implementation
//!
//! This module provides the bridge between Hyper's HTTP layer and Ruxno's
//! domain layer. It handles:
//!
//! - Converting Hyper requests to domain requests
//! - Dispatching through the App
//! - Converting domain responses back to Hyper responses
//!
//! # Architecture
//!
//! The service uses Hyper's `service_fn` pattern rather than implementing
//! `tower::Service` directly, which is simpler and more idiomatic for Hyper 1.x.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::server::RuxnoService;
//! use ruxno::app::App;
//! use std::sync::Arc;
//!
//! let app = App::new();
//! let service = RuxnoService::new(Arc::new(app));
//! ```

use crate::app::App;
use crate::core::CoreError;
use crate::http::{from_hyper_request, to_hyper_response};
use bytes::Bytes;
use hyper::body::Incoming;
use std::sync::Arc;

/// Ruxno service for Hyper
///
/// Wraps a Ruxno `App` and provides request handling for Hyper's HTTP server.
/// This service is cloneable and can be shared across multiple connections.
pub struct RuxnoService<E = ()> {
    app: Arc<App<E>>,
    production_mode: bool,
}

impl<E> Clone for RuxnoService<E> {
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
            production_mode: self.production_mode,
        }
    }
}

impl<E> RuxnoService<E>
where
    E: Send + Sync + 'static,
{
    /// Create new service from an App
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::server::RuxnoService;
    /// use ruxno::app::App;
    /// use std::sync::Arc;
    ///
    /// let app = App::new();
    /// let service = RuxnoService::new(Arc::new(app), false);
    /// ```
    pub fn new(app: Arc<App<E>>, production_mode: bool) -> Self {
        Self {
            app,
            production_mode,
        }
    }

    /// Handle a single HTTP request
    ///
    /// This is the core request handling method that:
    /// 1. Converts Hyper request to domain request (with body size and header limits)
    /// 2. Dispatches through the App (routing + middleware + handler)
    /// 3. Converts domain response back to Hyper response
    /// 4. Handles errors by converting them to HTTP error responses
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming Hyper HTTP request
    /// * `max_body_size` - Maximum allowed body size in bytes
    /// * `max_headers` - Maximum allowed number of headers
    ///
    /// # Returns
    ///
    /// Returns a Hyper response. This method never fails (returns `Infallible`)
    /// because all errors are converted to HTTP error responses.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = service.handle(hyper_request, 1024 * 1024, 100).await; // 1MB, 100 headers
    /// ```
    pub async fn handle(
        &self,
        req: hyper::Request<Incoming>,
        max_body_size: usize,
        max_headers: usize,
    ) -> Result<hyper::Response<http_body_util::Full<Bytes>>, std::convert::Infallible> {
        // Convert Hyper request to domain request with size and header limits
        let domain_req = match from_hyper_request(req, max_body_size, max_headers).await {
            Ok(req) => req,
            Err(err) => {
                // Body size limit exceeded, too many headers, or other conversion error
                let error_response = error_to_response(err, self.production_mode);
                return Ok(to_hyper_response(error_response));
            }
        };

        // Dispatch through app
        let domain_res = match self.app.dispatch(domain_req).await {
            Ok(res) => res,
            Err(err) => {
                // Convert error to HTTP response
                error_to_response(err, self.production_mode)
            }
        };

        // Convert domain response to Hyper response
        Ok(to_hyper_response(domain_res))
    }

    /// Get a reference to the underlying App
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let app_ref = service.app();
    /// ```
    pub fn app(&self) -> &Arc<App<E>> {
        &self.app
    }
}

/// Convert a CoreError to an HTTP Response
///
/// Maps error types to appropriate HTTP status codes and error messages.
/// In production mode, hides internal error details for 5xx errors.
///
/// # Arguments
///
/// * `err` - The error to convert
/// * `production_mode` - Whether to hide internal error details
///
/// # Security
///
/// In production mode:
/// - 5xx errors return generic messages (prevents information disclosure)
/// - Full error details are logged server-side only
/// - Error IDs are generated for correlation between logs and responses
///
/// In development mode:
/// - All error details are included in responses (for debugging)
fn error_to_response(err: CoreError, production_mode: bool) -> crate::domain::Response {
    use crate::core::StatusCode;
    use crate::domain::Response;

    let status = match &err {
        CoreError::NotFound(_) => StatusCode::NOT_FOUND,
        CoreError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
        CoreError::BadRequest(_) => StatusCode::BAD_REQUEST,
        CoreError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
        CoreError::RequestHeaderFieldsTooLarge(_) => StatusCode::from_u16(431).unwrap(),
        CoreError::InvalidPattern(_)
        | CoreError::DuplicateRoute { .. }
        | CoreError::MissingParameter(_)
        | CoreError::InvalidParameter { .. }
        | CoreError::BodyParseError(_) => StatusCode::BAD_REQUEST,
        CoreError::Internal(_) | CoreError::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Generate error ID for correlation
    let error_id = generate_error_id();

    // Determine if this is a server error (5xx)
    let is_server_error = err.is_server_error();

    // In production mode, hide internal error details for 5xx errors
    let error_message = if production_mode && is_server_error {
        // Log full error server-side
        eprintln!("🔴 [{}] Internal error: {}", error_id, err);

        // Return generic message to client
        "Internal Server Error".to_string()
    } else {
        // Development mode or client error - include details
        if is_server_error {
            // Log server errors even in development
            eprintln!("🔴 [{}] Internal error: {}", error_id, err);
        }
        err.to_string()
    };

    // Create error response with JSON body
    let error_body = serde_json::json!({
        "error": error_message,
        "status": status.as_u16(),
        "error_id": error_id,
    });

    Response::json(&error_body).with_status(status)
}

/// Generate a unique error ID for correlation
///
/// Uses timestamp + random component for uniqueness.
/// Format: `ERR-{timestamp}-{random}`
fn generate_error_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let random: u32 = rand::random();

    format!("ERR-{:x}-{:04x}", timestamp, random & 0xFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::core::StatusCode;

    #[tokio::test]
    async fn test_service_creation() {
        let app = App::new();
        let service = RuxnoService::new(Arc::new(app), false);
        // Service should be created successfully
        assert!(Arc::strong_count(service.app()) >= 1);
    }

    #[tokio::test]
    async fn test_service_clone() {
        let app = App::new();
        let service1 = RuxnoService::new(Arc::new(app), false);
        let service2 = service1.clone();

        // Both services should point to the same app
        assert!(Arc::ptr_eq(service1.app(), service2.app()));
    }

    #[tokio::test]
    async fn test_error_to_response_not_found() {
        let err = CoreError::NotFound("Route not found".to_string());
        let response = error_to_response(err, false);

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_error_to_response_bad_request() {
        let err = CoreError::BadRequest("Invalid input".to_string());
        let response = error_to_response(err, false);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_to_response_method_not_allowed() {
        let err = CoreError::MethodNotAllowed("POST not allowed".to_string());
        let response = error_to_response(err, false);

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_error_to_response_internal() {
        let err = CoreError::Internal("Server error".to_string());
        let response = error_to_response(err, false);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_to_response_payload_too_large() {
        let err = CoreError::PayloadTooLarge("Body too large".to_string());
        let response = error_to_response(err, false);

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_error_to_response_json_format() {
        let err = CoreError::NotFound("Resource not found".to_string());
        let response = error_to_response(err, false);

        // Check that content-type is JSON
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[tokio::test]
    async fn test_service_app_accessor() {
        let app = App::new();
        let app_arc = Arc::new(app);
        let service = RuxnoService::new(Arc::clone(&app_arc), false);

        assert!(Arc::ptr_eq(service.app(), &app_arc));
    }

    #[test]
    fn test_error_mapping_invalid_pattern() {
        let err = CoreError::InvalidPattern("Bad pattern".to_string());
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_duplicate_route() {
        let err = CoreError::DuplicateRoute {
            method: "GET".to_string(),
            path: "/users".to_string(),
        };
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_missing_parameter() {
        let err = CoreError::MissingParameter("id".to_string());
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_invalid_parameter() {
        let err = CoreError::InvalidParameter {
            name: "id".to_string(),
            reason: "not a number".to_string(),
        };
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_body_parse_error() {
        let err = CoreError::BodyParseError("Invalid JSON".to_string());
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_custom() {
        let err = CoreError::Custom("Custom error".to_string());
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_mapping_request_header_fields_too_large() {
        let err = CoreError::RequestHeaderFieldsTooLarge("Too many headers".to_string());
        let response = error_to_response(err, false);
        assert_eq!(response.status(), StatusCode::from_u16(431).unwrap());
    }

    #[test]
    fn test_production_mode_hides_internal_errors() {
        let err = CoreError::Internal("Database connection failed: password=secret123".to_string());
        let response = error_to_response(err, true); // Production mode

        // Response should have generic message
        let body = response.into_body();
        if let crate::domain::ResponseBody::Bytes(bytes) = body {
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(json["error"], "Internal Server Error");
            assert!(json["error_id"].is_string());
            // Should NOT contain sensitive details
            assert!(!json["error"].as_str().unwrap().contains("password"));
        }
    }

    #[test]
    fn test_development_mode_shows_error_details() {
        let err = CoreError::Internal("Database connection failed".to_string());
        let response = error_to_response(err, false); // Development mode

        // Response should have detailed message
        let body = response.into_body();
        if let crate::domain::ResponseBody::Bytes(bytes) = body {
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert!(
                json["error"]
                    .as_str()
                    .unwrap()
                    .contains("Database connection failed")
            );
            assert!(json["error_id"].is_string());
        }
    }

    #[test]
    fn test_client_errors_always_show_details() {
        let err = CoreError::BadRequest("Invalid email format".to_string());
        let response_prod = error_to_response(err.clone(), true);
        let response_dev = error_to_response(err, false);

        // Both should show details for client errors
        let body_prod = response_prod.into_body();
        let body_dev = response_dev.into_body();

        if let crate::domain::ResponseBody::Bytes(bytes) = body_prod {
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert!(
                json["error"]
                    .as_str()
                    .unwrap()
                    .contains("Invalid email format")
            );
        }

        if let crate::domain::ResponseBody::Bytes(bytes) = body_dev {
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert!(
                json["error"]
                    .as_str()
                    .unwrap()
                    .contains("Invalid email format")
            );
        }
    }

    #[test]
    fn test_error_id_format() {
        let err = CoreError::Internal("Test error".to_string());
        let response = error_to_response(err, false);

        let body = response.into_body();
        if let crate::domain::ResponseBody::Bytes(bytes) = body {
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let error_id = json["error_id"].as_str().unwrap();

            // Should match format: ERR-{hex}-{hex}
            assert!(error_id.starts_with("ERR-"));
            assert!(error_id.contains('-'));
            assert!(error_id.len() > 10); // Reasonable length check
        }
    }
}
