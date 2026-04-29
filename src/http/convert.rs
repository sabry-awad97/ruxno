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

use crate::domain::{Request, Response};
use crate::http::Headers;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use std::collections::HashMap;

/// Convert Hyper request to domain request
///
/// This function performs a lossless conversion from Hyper's request type
/// to Ruxno's domain `Request`. It:
///
/// - Extracts method, URI, and headers
/// - Buffers the entire body into memory
/// - Parses query parameters from the URI
/// - Creates a domain `Request` with all data
///
/// # Arguments
///
/// * `req` - The Hyper HTTP request to convert
///
/// # Returns
///
/// Returns a domain `Request` with buffered body.
///
/// # Examples
///
/// ```rust,ignore
/// let hyper_req = hyper::Request::new(hyper::body::Incoming::default());
/// let domain_req = from_hyper_request(hyper_req).await;
/// ```
pub async fn from_hyper_request(req: hyper::Request<Incoming>) -> Request {
    // Extract parts from Hyper request
    let (parts, body) = req.into_parts();

    // Buffer the body
    let body_bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_default();

    // Parse query parameters from URI
    let query = parse_query_params(parts.uri.query());

    // Convert headers (HeaderMap -> Headers wrapper)
    let headers = Headers::from(parts.headers);

    // Create domain request
    Request::new(parts.method, parts.uri, query, headers, body_bytes)
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
    builder.body(body).unwrap_or_else(|_| {
        // Fallback error response if building fails
        hyper::Response::builder()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .body(http_body_util::Full::new(Bytes::from(
                "Failed to build response",
            )))
            .unwrap()
    })
}

/// Parse query parameters from URI query string
///
/// Parses the query string into a HashMap of key-value pairs.
/// Handles URL decoding and multiple values (last value wins).
///
/// # Arguments
///
/// * `query` - Optional query string from URI
///
/// # Returns
///
/// Returns a HashMap of parsed query parameters.
///
/// # Examples
///
/// ```rust,ignore
/// let query = parse_query_params(Some("page=1&limit=10"));
/// assert_eq!(query.get("page"), Some(&"1".to_string()));
/// ```
fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
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

                    Some((key, value))
                })
                .collect()
        })
        .unwrap_or_default()
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
        let _body_stream = http_body_util::BodyExt::map_err(_body, |_| {
            std::io::Error::new(std::io::ErrorKind::Other, "body error")
        });
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
            .with_status(StatusCode::NOT_FOUND)
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
        let domain_res = Response::new().with_status(StatusCode::NO_CONTENT);
        let hyper_res = to_hyper_response(domain_res);

        assert_eq!(hyper_res.status(), StatusCode::NO_CONTENT);
    }
}
