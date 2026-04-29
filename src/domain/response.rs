//! Response domain model
//!
//! This module defines the `Response` type, which represents an HTTP response
//! in a protocol-agnostic way. The response uses a builder pattern for
//! ergonomic construction.
//!
//! # Design
//!
//! - **Builder pattern**: Fluent API for constructing responses
//! - **HTTP-aware**: Uses `http::StatusCode` and `http::HeaderMap`
//! - **Flexible body**: Supports empty, bytes, and streaming bodies
//! - **Convenience methods**: `text()`, `json()`, `html()`, `redirect()`
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::Response;
//!
//! // Text response
//! let res = Response::text("Hello, World!");
//!
//! // JSON response
//! let res = Response::json(&serde_json::json!({ "status": "ok" }));
//!
//! // Custom response with builder pattern
//! let res = Response::new()
//!     .with_status(201)
//!     .with_header("x-custom", "value")
//!     .with_body("Created");
//!
//! // Redirect
//! let res = Response::redirect("/login");
//! ```

use crate::core::StatusCode;
use bytes::Bytes;
use futures_util::Stream;
use http::HeaderMap;
use std::pin::Pin;

/// Response body type
///
/// Represents different types of response bodies:
/// - `Empty`: No body content (e.g., 204 No Content)
/// - `Bytes`: Static bytes (most common case)
/// - `Stream`: Streaming body for large responses or SSE
pub enum ResponseBody {
    /// Empty body (no content)
    Empty,

    /// Static bytes (buffered in memory)
    Bytes(Bytes),

    /// Streaming body (for large responses, SSE, etc.)
    ///
    /// Note: Streams are not cloneable. If you need to clone a response,
    /// use `Empty` or `Bytes` variants.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>),
}

impl ResponseBody {
    /// Check if body is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, ResponseBody::Empty)
    }

    /// Get body size if known (None for streams)
    pub fn size(&self) -> Option<usize> {
        match self {
            ResponseBody::Empty => Some(0),
            ResponseBody::Bytes(bytes) => Some(bytes.len()),
            ResponseBody::Stream(_) => None,
        }
    }
}

/// Response domain model
///
/// Represents an HTTP response with status, headers, and body.
/// Uses builder pattern for ergonomic construction.
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::Response;
///
/// // Simple text response
/// let res = Response::text("Hello");
///
/// // JSON response
/// let res = Response::json(&serde_json::json!({ "id": 123 }));
///
/// // Custom response
/// let res = Response::new()
///     .with_status(201)
///     .with_header("location", "/users/123")
///     .with_body("Created");
/// ```
pub struct Response {
    /// HTTP status code
    status: StatusCode,

    /// Response headers (HTTP-aware, case-insensitive)
    headers: HeaderMap,

    /// Response body
    body: ResponseBody,
}

impl Response {
    /// Create a new empty response with 200 OK status
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::new();
    /// assert_eq!(res.status(), StatusCode::OK);
    /// ```
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: ResponseBody::Empty,
        }
    }

    /// Create a text response with 200 OK status
    ///
    /// Sets `Content-Type: text/plain; charset=utf-8` header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::text("Hello, World!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/plain; charset=utf-8".parse().unwrap());

        Self {
            status: StatusCode::OK,
            headers,
            body: ResponseBody::Bytes(Bytes::from(text.into())),
        }
    }

    /// Create a JSON response with 200 OK status
    ///
    /// Sets `Content-Type: application/json` header.
    /// Returns 500 Internal Server Error if serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::json(&serde_json::json!({ "status": "ok" }));
    /// ```
    pub fn json<T: serde::Serialize>(value: &T) -> Self {
        match serde_json::to_string(value) {
            Ok(body) => {
                let mut headers = HeaderMap::new();
                headers.insert("content-type", "application/json".parse().unwrap());

                Self {
                    status: StatusCode::OK,
                    headers,
                    body: ResponseBody::Bytes(Bytes::from(body)),
                }
            }
            Err(_) => {
                // Return 500 error if serialization fails
                Self::new()
                    .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_body("Failed to serialize JSON")
            }
        }
    }

    /// Create an HTML response with 200 OK status
    ///
    /// Sets `Content-Type: text/html; charset=utf-8` header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::html("<h1>Hello</h1>");
    /// ```
    pub fn html(html: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "text/html; charset=utf-8".parse().unwrap());

        Self {
            status: StatusCode::OK,
            headers,
            body: ResponseBody::Bytes(Bytes::from(html.into())),
        }
    }

    /// Create a redirect response
    ///
    /// Returns 302 Found with `Location` header.
    /// Use `with_status()` to change to 301 (permanent) or 307/308 (preserve method).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Temporary redirect (302)
    /// let res = Response::redirect("/login");
    ///
    /// // Permanent redirect (301)
    /// let res = Response::redirect("/new-url").with_status(StatusCode::MOVED_PERMANENTLY);
    /// ```
    pub fn redirect(location: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("location", location.into().parse().unwrap());

        Self {
            status: StatusCode::FOUND, // 302
            headers,
            body: ResponseBody::Empty,
        }
    }

    /// Set status code (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::new().with_status(StatusCode::CREATED);
    /// ```
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a header (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::new()
    ///     .with_header("x-request-id", "123")
    ///     .with_header("cache-control", "no-cache");
    /// ```
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name_str = name.into();
        let value_str = value.into();

        // Try to parse both name and value as valid HTTP header components
        if let (Ok(header_name), Ok(header_value)) = (
            name_str.parse::<http::header::HeaderName>(),
            value_str.parse::<http::header::HeaderValue>(),
        ) {
            self.headers.insert(header_name, header_value);
        }
        self
    }

    /// Set body (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::new().with_body("Hello, World!");
    /// ```
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = ResponseBody::Bytes(Bytes::from(body.into()));
        self
    }

    /// Set body as bytes (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let res = Response::new().with_bytes(Bytes::from("data"));
    /// ```
    pub fn with_bytes(mut self, bytes: Bytes) -> Self {
        self.body = ResponseBody::Bytes(bytes);
        self
    }

    /// Get status code
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(res.status(), StatusCode::OK);
    /// ```
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = res.headers();
    /// ```
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get mutable headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// res.headers_mut().insert("x-custom", "value".parse().unwrap());
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get body
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = res.body();
    /// ```
    pub fn body(&self) -> &ResponseBody {
        &self.body
    }

    /// Take body (consumes self)
    ///
    /// Useful for converting to HTTP response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let body = res.into_body();
    /// ```
    pub fn into_body(self) -> ResponseBody {
        self.body
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience conversions

impl From<String> for Response {
    fn from(text: String) -> Self {
        Response::text(text)
    }
}

impl From<&str> for Response {
    fn from(text: &str) -> Self {
        Response::text(text)
    }
}

impl From<Bytes> for Response {
    fn from(bytes: Bytes) -> Self {
        Response::new().with_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new() {
        let res = Response::new();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.body().is_empty());
        assert_eq!(res.headers().len(), 0);
    }

    #[test]
    fn test_response_text() {
        let res = Response::text("Hello, World!");
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Hello, World!"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_json() {
        let data = serde_json::json!({ "name": "John", "age": 30 });
        let res = Response::json(&data);
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json"
        );
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                let parsed: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                assert_eq!(parsed["name"], "John");
                assert_eq!(parsed["age"], 30);
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_html() {
        let res = Response::html("<h1>Hello</h1>");
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("<h1>Hello</h1>"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_redirect() {
        let res = Response::redirect("/login");
        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(res.headers().get("location").unwrap(), "/login");
        assert!(res.body().is_empty());
    }

    #[test]
    fn test_response_redirect_permanent() {
        let res = Response::redirect("/new-url").with_status(StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.headers().get("location").unwrap(), "/new-url");
    }

    #[test]
    fn test_response_with_status() {
        let res = Response::new().with_status(StatusCode::CREATED);
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[test]
    fn test_response_with_header() {
        let res = Response::new()
            .with_header("x-request-id", "123")
            .with_header("cache-control", "no-cache");

        assert_eq!(res.headers().get("x-request-id").unwrap(), "123");
        assert_eq!(res.headers().get("cache-control").unwrap(), "no-cache");
    }

    #[test]
    fn test_response_with_body() {
        let res = Response::new().with_body("Custom body");
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Custom body"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_with_bytes() {
        let data = Bytes::from("binary data");
        let res = Response::new().with_bytes(data.clone());
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &data);
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_builder_pattern() {
        let res = Response::new()
            .with_status(StatusCode::CREATED)
            .with_header("location", "/users/123")
            .with_header("x-custom", "value")
            .with_body("User created");

        assert_eq!(res.status(), StatusCode::CREATED);
        assert_eq!(res.headers().get("location").unwrap(), "/users/123");
        assert_eq!(res.headers().get("x-custom").unwrap(), "value");
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("User created"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_from_string() {
        let res: Response = "Hello".to_string().into();
        assert_eq!(res.status(), StatusCode::OK);
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Hello"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_from_str() {
        let res: Response = "Hello".into();
        assert_eq!(res.status(), StatusCode::OK);
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Hello"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_from_bytes() {
        let data = Bytes::from("data");
        let res: Response = data.clone().into();
        match res.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &data);
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_body_is_empty() {
        assert!(ResponseBody::Empty.is_empty());
        assert!(!ResponseBody::Bytes(Bytes::from("data")).is_empty());
    }

    #[test]
    fn test_response_body_size() {
        assert_eq!(ResponseBody::Empty.size(), Some(0));
        assert_eq!(ResponseBody::Bytes(Bytes::from("hello")).size(), Some(5));
    }

    #[test]
    fn test_response_headers_mut() {
        let mut res = Response::new();
        res.headers_mut()
            .insert("x-custom", "value".parse().unwrap());
        assert_eq!(res.headers().get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_response_into_body() {
        let res = Response::text("Hello");
        let body = res.into_body();
        match body {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, Bytes::from("Hello"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[test]
    fn test_response_default() {
        let res = Response::default();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.body().is_empty());
    }
}
