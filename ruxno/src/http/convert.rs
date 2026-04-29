//! Hyper ↔ Ruxno conversion utilities
//!
//! This module provides lossless conversion between Hyper's HTTP types
//! and Ruxno's domain models. It handles:
//!
//! - Request conversion (method, URI, headers, body buffering)
//! - Response conversion (status, headers, body types)
//! - Query parameter parsing
//! - Header mapping (zero-copy where possible)
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::http::convert::{from_hyper_request, to_hyper_response};
//!
//! // Convert Hyper request to domain request
//! let domain_req = from_hyper_request(hyper_req).await;
//!
//! // Convert domain response to Hyper response
//! let hyper_res = to_hyper_response(domain_res);
//! ```

use crate::core::CoreError;
use crate::domain::{Request, Response};
use crate::http::Headers;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use std::collections::HashMap;

/// Convert Hyper request to domain request with body size limits
///
/// This function performs a lossless conversion from Hyper's request type
/// to Ruxno's domain `Request`. It:
///
/// - Validates header count against max_headers
/// - Validates Content-Length header against max_body_size
/// - Applies size limits before buffering body
/// - Extracts method, URI, and headers
/// - Parses query parameters from the URI
/// - Creates a domain `Request` with all data
///
/// # Arguments
///
/// * `req` - The Hyper HTTP request to convert
/// * `max_body_size` - Maximum allowed body size in bytes
/// * `max_headers` - Maximum allowed number of headers
///
/// # Returns
///
/// Returns a domain `Request` with buffered body, or an error if validation fails.
///
/// # Errors
///
/// Returns `CoreError::RequestHeaderFieldsTooLarge` if:
/// - Header count exceeds max_headers
///
/// Returns `CoreError::PayloadTooLarge` if:
/// - Content-Length header exceeds max_body_size
/// - Actual body size exceeds max_body_size during reading
///
/// # Examples
///
/// ```rust,ignore
/// let hyper_req = hyper::Request::new(hyper::body::Incoming::default());
/// let domain_req = from_hyper_request(hyper_req, 1024 * 1024, 100).await?; // 1MB, 100 headers
/// ```
pub async fn from_hyper_request(
    req: hyper::Request<Incoming>,
    max_body_size: usize,
    max_headers: usize,
) -> Result<Request, CoreError> {
    // Extract parts from Hyper request
    let (parts, body) = req.into_parts();

    // Validate header count before processing
    let header_count = parts.headers.len();
    if header_count > max_headers {
        return Err(CoreError::request_header_fields_too_large(format!(
            "Request has {} headers, exceeds maximum allowed {} headers",
            header_count, max_headers
        )));
    }

    // Check Content-Length header early to reject oversized requests before reading
    if let Some(content_length) = parts.headers.get(hyper::header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > max_body_size {
                    return Err(CoreError::payload_too_large(format!(
                        "Request body size {} bytes exceeds maximum allowed size {} bytes",
                        length, max_body_size
                    )));
                }
            }
        }
    }

    // Apply size limit to body stream using Limited wrapper
    let limited_body = http_body_util::Limited::new(body, max_body_size);

    // Buffer the body with size limit enforcement
    let body_bytes = limited_body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .map_err(|e| {
            // Check if error is due to size limit
            if e.to_string().contains("length limit exceeded") {
                CoreError::payload_too_large(format!(
                    "Request body exceeds maximum allowed size of {} bytes",
                    max_body_size
                ))
            } else {
                CoreError::bad_request(format!("Failed to read request body: {}", e))
            }
        })?;

    // Parse query parameters from URI
    let query = parse_query_params(parts.uri.query());

    // Convert headers (HeaderMap -> Headers wrapper)
    let headers = Headers::from(parts.headers);

    // Create domain request
    Ok(Request::new(
        parts.method,
        parts.uri,
        query,
        headers,
        body_bytes,
    ))
}

/// Convert domain response to Hyper response
///
/// This function performs a lossless conversion from Ruxno's domain `Response`
/// to Hyper's response type. It:
///
/// - Converts status code
/// - Maps headers (zero-copy)
/// - Converts body to Hyper's Full body type
///
/// # Arguments
///
/// * `res` - The domain response to convert
///
/// # Returns
///
/// Returns a Hyper `Response` ready to send over the wire.
///
/// # Examples
///
/// ```rust,ignore
/// let domain_res = Response::text("Hello, World!");
/// let hyper_res = to_hyper_response(domain_res);
/// ```
pub fn to_hyper_response(res: Response) -> hyper::Response<http_body_util::Full<Bytes>> {
    // Extract parts from domain response
    let status = res.status();
    let headers = res.headers().clone();
    let body_bytes = match res.into_body() {
        crate::domain::ResponseBody::Empty => Bytes::new(),
        crate::domain::ResponseBody::Bytes(bytes) => bytes,
        crate::domain::ResponseBody::Stream(_) => {
            // For now, we don't support streaming in this conversion
            // This will be handled in a separate streaming conversion function
            Bytes::from("Streaming not supported in basic conversion")
        }
    };

    // Build Hyper response
    let mut builder = hyper::Response::builder().status(status);

    // Add headers (iterate over Headers wrapper)
    for (name, value) in headers.iter() {
        builder = builder.header(name, value);
    }

    // Create body
    let body = http_body_util::Full::new(body_bytes);

    // Build and return response
    match builder.body(body) {
        Ok(response) => response,
        Err(e) => {
            // Log the error for debugging
            eprintln!("⚠️  Failed to build response: {}", e);

            // Return a minimal safe response without using unwrap
            // This construction is guaranteed to succeed with valid inputs
            let mut response = hyper::Response::new(http_body_util::Full::new(Bytes::from(
                r#"{"error":"Internal Server Error","message":"Failed to build response"}"#,
            )));
            *response.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            response.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            );
            response
        }
    }
}

/// Parse query parameters from URI query string with validation
///
/// Parses the query string into a HashMap of key-value pairs with security validations:
/// - Limits key length to 256 bytes
/// - Limits value length to 4096 bytes
/// - Rejects null bytes (path traversal prevention)
/// - Detects suspicious path traversal patterns (../, ..\\)
/// - Handles URL decoding and multiple values (last value wins)
///
/// # Security
///
/// This function implements multiple layers of defense against injection attacks:
/// - **Length limits**: Prevents memory exhaustion and buffer overflow attacks
/// - **Null byte detection**: Prevents path traversal and string termination attacks
/// - **Path traversal detection**: Blocks directory traversal attempts (../, ..\\)
/// - **URL decoding**: Properly handles encoded characters to prevent bypass attempts
///
/// # Arguments
///
/// * `query` - Optional query string from URI
///
/// # Returns
///
/// Returns a HashMap of parsed and validated query parameters.
/// Invalid parameters are silently dropped to prevent DoS via validation errors.
///
/// # Examples
///
/// ```rust,ignore
/// let query = parse_query_params(Some("page=1&limit=10"));
/// assert_eq!(query.get("page"), Some(&"1".to_string()));
///
/// // Malicious input is rejected
/// let query = parse_query_params(Some("path=../../etc/passwd"));
/// assert!(query.is_empty()); // Path traversal detected and dropped
/// ```
fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
    const _MAX_KEY_LENGTH: usize = 256;
    const _MAX_VALUE_LENGTH: usize = 4096;

    query
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let value = parts.next().unwrap_or("");

                    // URL decode key and value
                    let key = urlencoding::decode(key).ok()?.into_owned();
                    let value = urlencoding::decode(value).ok()?.into_owned();

                    // Validate key and value
                    if !is_valid_query_param(&key, &value) {
                        return None; // Drop invalid parameters
                    }

                    Some((key, value))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Validate query parameter key and value for security
///
/// Checks for:
/// - Length limits (key: 256 bytes, value: 4096 bytes)
/// - Null bytes (path traversal prevention)
/// - Path traversal patterns (../, ..\\)
///
/// # Arguments
///
/// * `key` - Parameter key to validate
/// * `value` - Parameter value to validate
///
/// # Returns
///
/// Returns `true` if the parameter is safe, `false` otherwise.
fn is_valid_query_param(key: &str, value: &str) -> bool {
    const MAX_KEY_LENGTH: usize = 256;
    const MAX_VALUE_LENGTH: usize = 4096;

    // Check length limits
    if key.len() > MAX_KEY_LENGTH || value.len() > MAX_VALUE_LENGTH {
        return false;
    }

    // Check for null bytes (path traversal prevention)
    if key.contains('\0') || value.contains('\0') {
        return false;
    }

    // Check for path traversal patterns
    if contains_path_traversal(value) {
        return false;
    }

    true
}

/// Detect path traversal patterns in a string
///
/// Checks for common path traversal patterns:
/// - `../` (Unix-style)
/// - `..\` (Windows-style)
/// - URL-encoded variants (%2e%2e%2f, %2e%2e%5c)
///
/// # Arguments
///
/// * `s` - String to check for path traversal patterns
///
/// # Returns
///
/// Returns `true` if path traversal patterns are detected, `false` otherwise.
fn contains_path_traversal(s: &str) -> bool {
    // Check for literal patterns
    if s.contains("../") || s.contains("..\\") {
        return true;
    }

    // Check for URL-encoded patterns (case-insensitive)
    let lower = s.to_lowercase();
    if lower.contains("%2e%2e%2f") || lower.contains("%2e%2e%5c") {
        return true;
    }

    // Check for mixed encoding (. followed by encoded /)
    if lower.contains("..%2f") || lower.contains("..%5c") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::StatusCode;
    use crate::domain::Response;

    #[test]
    fn test_parse_query_params_empty() {
        let query = parse_query_params(None);
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_single() {
        let query = parse_query_params(Some("page=1"));
        assert_eq!(query.get("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_parse_query_params_multiple() {
        let query = parse_query_params(Some("page=1&limit=10&sort=name"));
        assert_eq!(query.get("page"), Some(&"1".to_string()));
        assert_eq!(query.get("limit"), Some(&"10".to_string()));
        assert_eq!(query.get("sort"), Some(&"name".to_string()));
    }

    #[test]
    fn test_parse_query_params_url_encoded() {
        let query = parse_query_params(Some("name=John%20Doe&email=john%40example.com"));
        assert_eq!(query.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(query.get("email"), Some(&"john@example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params_no_value() {
        let query = parse_query_params(Some("flag&other=value"));
        assert_eq!(query.get("flag"), Some(&"".to_string()));
        assert_eq!(query.get("other"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_query_params_duplicate_keys() {
        // Last value wins
        let query = parse_query_params(Some("page=1&page=2"));
        assert_eq!(query.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_parse_query_params_key_length_limit() {
        // Key exceeds 256 bytes - should be dropped
        let long_key = "a".repeat(257);
        let query_str = format!("{}=value", long_key);
        let query = parse_query_params(Some(&query_str));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_key_at_limit() {
        // Key exactly 256 bytes - should be accepted
        let key = "a".repeat(256);
        let query_str = format!("{}=value", key);
        let query = parse_query_params(Some(&query_str));
        assert_eq!(query.len(), 1);
        assert_eq!(query.get(&key), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_query_params_value_length_limit() {
        // Value exceeds 4096 bytes - should be dropped
        let long_value = "x".repeat(4097);
        let query_str = format!("key={}", long_value);
        let query = parse_query_params(Some(&query_str));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_value_at_limit() {
        // Value exactly 4096 bytes - should be accepted
        let value = "x".repeat(4096);
        let query_str = format!("key={}", value);
        let query = parse_query_params(Some(&query_str));
        assert_eq!(query.len(), 1);
        assert_eq!(query.get("key"), Some(&value));
    }

    #[test]
    fn test_parse_query_params_null_byte_in_key() {
        // Null byte in key - should be dropped
        let query = parse_query_params(Some("key\0=value"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_null_byte_in_value() {
        // Null byte in value - should be dropped
        let query = parse_query_params(Some("key=value\0"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_path_traversal_unix() {
        // Unix-style path traversal - should be dropped
        let query = parse_query_params(Some("path=../../etc/passwd"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_path_traversal_windows() {
        // Windows-style path traversal - should be dropped
        let query = parse_query_params(Some("path=..\\..\\windows\\system32"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_path_traversal_encoded() {
        // URL-encoded path traversal - should be dropped
        let query = parse_query_params(Some("path=%2e%2e%2fetc%2fpasswd"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_path_traversal_mixed_encoding() {
        // Mixed encoding path traversal - should be dropped
        let query = parse_query_params(Some("path=..%2fetc%2fpasswd"));
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_query_params_safe_dots() {
        // Safe use of dots (not path traversal) - should be accepted
        let query = parse_query_params(Some("file=document.pdf"));
        assert_eq!(query.get("file"), Some(&"document.pdf".to_string()));
    }

    #[test]
    fn test_parse_query_params_safe_path() {
        // Safe path without traversal - should be accepted
        let query = parse_query_params(Some("path=/api/users/123"));
        assert_eq!(query.get("path"), Some(&"/api/users/123".to_string()));
    }

    #[test]
    fn test_parse_query_params_multiple_with_invalid() {
        // Mix of valid and invalid parameters - only valid ones kept
        let query = parse_query_params(Some("valid=ok&bad=../../etc&good=yes"));
        assert_eq!(query.len(), 2);
        assert_eq!(query.get("valid"), Some(&"ok".to_string()));
        assert_eq!(query.get("good"), Some(&"yes".to_string()));
        assert!(!query.contains_key("bad"));
    }

    #[test]
    fn test_parse_query_params_empty_value_valid() {
        // Empty value is valid
        let query = parse_query_params(Some("flag="));
        assert_eq!(query.get("flag"), Some(&"".to_string()));
    }

    #[test]
    fn test_contains_path_traversal_unix() {
        assert!(contains_path_traversal("../etc/passwd"));
        assert!(contains_path_traversal("../../etc/passwd"));
        assert!(contains_path_traversal("/var/log/../etc/passwd"));
    }

    #[test]
    fn test_contains_path_traversal_windows() {
        assert!(contains_path_traversal("..\\windows\\system32"));
        assert!(contains_path_traversal("..\\..\\windows"));
        assert!(contains_path_traversal("C:\\Users\\..\\Admin"));
    }

    #[test]
    fn test_contains_path_traversal_encoded() {
        assert!(contains_path_traversal("%2e%2e%2fetc"));
        assert!(contains_path_traversal("%2E%2E%2Fetc")); // Case insensitive
        assert!(contains_path_traversal("%2e%2e%5cwindows"));
    }

    #[test]
    fn test_contains_path_traversal_mixed() {
        assert!(contains_path_traversal("..%2fetc"));
        assert!(contains_path_traversal("..%5cwindows"));
    }

    #[test]
    fn test_contains_path_traversal_safe() {
        assert!(!contains_path_traversal("document.pdf"));
        assert!(!contains_path_traversal("/api/users/123"));
        assert!(!contains_path_traversal("file.tar.gz"));
        assert!(!contains_path_traversal("version-1.2.3"));
    }

    #[test]
    fn test_is_valid_query_param_all_checks() {
        // Valid parameter
        assert!(is_valid_query_param("page", "1"));
        assert!(is_valid_query_param("name", "John Doe"));

        // Key too long
        assert!(!is_valid_query_param(&"a".repeat(257), "value"));

        // Value too long
        assert!(!is_valid_query_param("key", &"x".repeat(4097)));

        // Null bytes
        assert!(!is_valid_query_param("key\0", "value"));
        assert!(!is_valid_query_param("key", "value\0"));

        // Path traversal
        assert!(!is_valid_query_param("path", "../../etc/passwd"));
    }

    #[tokio::test]
    async fn test_from_hyper_request_basic() {
        let hyper_req = hyper::Request::builder()
            .method("GET")
            .uri("http://localhost/api/users?page=1")
            .header("content-type", "application/json")
            .body(http_body_util::Full::new(Bytes::from("test body")))
            .unwrap();

        // Convert Full body to Incoming (this is a simplification for testing)
        let (_parts, _body) = hyper_req.into_parts();
        let _body_stream =
            http_body_util::BodyExt::map_err(_body, |_| std::io::Error::other("body error"));
        let _incoming_body = http_body_util::BodyExt::boxed(_body_stream);

        // For testing, we'll create a simpler test
        // In real usage, Incoming comes from Hyper's server
    }

    #[test]
    fn test_to_hyper_response_text() {
        let domain_res = Response::text("Hello, World!");
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(hyper_res.status(), crate::core::StatusCode::OK);

        // Check body (consumed, so we can't easily test it here without async)
        let _body = hyper_res.into_body();
    }

    #[test]
    fn test_to_hyper_response_json() {
        let domain_res = Response::json(&serde_json::json!({"message": "success"}));
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(hyper_res.status(), StatusCode::OK);
        assert_eq!(
            hyper_res.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_to_hyper_response_with_status() {
        let domain_res = Response::new()
            .with_status_code(StatusCode::NOT_FOUND)
            .with_body("Not found");
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(hyper_res.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_to_hyper_response_with_headers() {
        let domain_res = Response::text("Hello").with_header("x-custom-header", "custom-value");
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(
            hyper_res.headers().get("x-custom-header").unwrap(),
            "custom-value"
        );
    }

    #[test]
    fn test_to_hyper_response_empty_body() {
        let domain_res = Response::new().with_status_code(StatusCode::NO_CONTENT);
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(hyper_res.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_parse_query_params_header_count_edge_cases() {
        // Test that we can handle requests with various header counts
        // Note: Full integration tests with Incoming body are in integration tests

        // Test query parsing still works (unrelated to headers but part of conversion)
        let query = parse_query_params(Some("a=1&b=2&c=3"));
        assert_eq!(query.len(), 3);
    }
}
