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
}

impl<E> Clone for RuxnoService<E> {
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
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
    /// let service = RuxnoService::new(Arc::new(app));
    /// ```
    pub fn new(app: Arc<App<E>>) -> Self {
        Self { app }
    }

    /// Handle a single HTTP request
    ///
    /// This is the core request handling method that:
    /// 1. Converts Hyper request to domain request
    /// 2. Dispatches through the App (routing + middleware + handler)
    /// 3. Converts domain response back to Hyper response
    /// 4. Handles errors by converting them to HTTP error responses
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming Hyper HTTP request
    ///
    /// # Returns
    ///
    /// Returns a Hyper response. This method never fails (returns `Infallible`)
    /// because all errors are converted to HTTP error responses.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = service.handle(hyper_request).await;
    /// ```
    pub async fn handle(
        &self,
        req: hyper::Request<Incoming>,
    ) -> Result<hyper::Response<http_body_util::Full<Bytes>>, std::convert::Infallible> {
        // Convert Hyper request to domain request
        let domain_req = from_hyper_request(req).await;

        // Dispatch through app
        let domain_res = match self.app.dispatch(domain_req).await {
            Ok(res) => res,
            Err(err) => {
                // Convert error to HTTP response
                error_to_response(err)
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
fn error_to_response(err: CoreError) -> crate::domain::Response {
    use crate::core::StatusCode;
    use crate::domain::Response;

    let status = match &err {
        CoreError::NotFound(_) => StatusCode::NOT_FOUND,
        CoreError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
        CoreError::BadRequest(_) => StatusCode::BAD_REQUEST,
        CoreError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
        CoreError::InvalidPattern(_)
        | CoreError::DuplicateRoute { .. }
        | CoreError::MissingParameter(_)
        | CoreError::InvalidParameter { .. }
        | CoreError::BodyParseError(_) => StatusCode::BAD_REQUEST,
        CoreError::Internal(_) | CoreError::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Create error response with JSON body
    let error_body = serde_json::json!({
        "error": err.to_string(),
        "status": status.as_u16(),
    });

    Response::json(&error_body).with_status(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::core::{Method, StatusCode};
    use crate::domain::{Context, Response};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_service_creation() {
        let app = App::new();
        let service = RuxnoService::new(Arc::new(app));
        // Service should be created successfully
        assert!(Arc::strong_count(service.app()) >= 1);
    }

    #[tokio::test]
    async fn test_service_clone() {
        let app = App::new();
        let service1 = RuxnoService::new(Arc::new(app));
        let service2 = service1.clone();

        // Both services should point to the same app
        assert!(Arc::ptr_eq(service1.app(), service2.app()));
    }

    #[tokio::test]
    async fn test_error_to_response_not_found() {
        let err = CoreError::NotFound("Route not found".to_string());
        let response = error_to_response(err);

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_error_to_response_bad_request() {
        let err = CoreError::BadRequest("Invalid input".to_string());
        let response = error_to_response(err);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_to_response_method_not_allowed() {
        let err = CoreError::MethodNotAllowed("POST not allowed".to_string());
        let response = error_to_response(err);

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_error_to_response_internal() {
        let err = CoreError::Internal("Server error".to_string());
        let response = error_to_response(err);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_to_response_payload_too_large() {
        let err = CoreError::PayloadTooLarge("Body too large".to_string());
        let response = error_to_response(err);

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_error_to_response_json_format() {
        let err = CoreError::NotFound("Resource not found".to_string());
        let response = error_to_response(err);

        // Check that content-type is JSON
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[tokio::test]
    async fn test_service_app_accessor() {
        let app = App::new();
        let app_arc = Arc::new(app);
        let service = RuxnoService::new(Arc::clone(&app_arc));

        assert!(Arc::ptr_eq(service.app(), &app_arc));
    }

    #[test]
    fn test_error_mapping_invalid_pattern() {
        let err = CoreError::InvalidPattern("Bad pattern".to_string());
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_duplicate_route() {
        let err = CoreError::DuplicateRoute {
            method: "GET".to_string(),
            path: "/users".to_string(),
        };
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_missing_parameter() {
        let err = CoreError::MissingParameter("id".to_string());
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_invalid_parameter() {
        let err = CoreError::InvalidParameter {
            name: "id".to_string(),
            reason: "not a number".to_string(),
        };
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_body_parse_error() {
        let err = CoreError::BodyParseError("Invalid JSON".to_string());
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_mapping_custom() {
        let err = CoreError::Custom("Custom error".to_string());
        let response = error_to_response(err);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
