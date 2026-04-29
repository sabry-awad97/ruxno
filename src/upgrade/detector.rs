//! Upgrade detection logic
//!
//! Provides unified detection for protocol upgrades (WebSocket, SSE) based on
//! request headers. This eliminates duplicate upgrade logic and enables
//! automatic protocol detection.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::upgrade::{UpgradeDetector, UpgradeType};
//! use ruxno::domain::Request;
//!
//! // Detect upgrade type
//! match UpgradeDetector::detect(&request) {
//!     Some(UpgradeType::WebSocket) => println!("WebSocket upgrade detected"),
//!     Some(UpgradeType::SSE) => println!("SSE upgrade detected"),
//!     None => println!("Regular HTTP request"),
//! }
//!
//! // Check specific upgrade types
//! if UpgradeDetector::is_websocket(&request) {
//!     println!("WebSocket upgrade");
//! }
//!
//! if UpgradeDetector::is_sse(&request) {
//!     println!("SSE request");
//! }
//! ```

use crate::domain::Request;
use crate::upgrade::UpgradeType;

/// Upgrade detector
///
/// Provides methods to detect protocol upgrades from HTTP requests.
/// Supports WebSocket and Server-Sent Events (SSE) detection.
///
/// # Design
///
/// Uses a zero-sized struct with static methods for stateless detection.
/// All detection logic is based on HTTP headers following RFC specifications.
pub struct UpgradeDetector;

impl UpgradeDetector {
    /// Detect upgrade type from request headers
    ///
    /// Returns `Some(UpgradeType)` if the request is a valid upgrade request,
    /// or `None` if it's a regular HTTP request.
    ///
    /// # Detection Priority
    ///
    /// 1. WebSocket - Checked first (requires explicit upgrade)
    /// 2. SSE - Checked second (content negotiation)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::upgrade::{UpgradeDetector, UpgradeType};
    ///
    /// // WebSocket request
    /// assert_eq!(
    ///     UpgradeDetector::detect(&ws_request),
    ///     Some(UpgradeType::WebSocket)
    /// );
    ///
    /// // SSE request
    /// assert_eq!(
    ///     UpgradeDetector::detect(&sse_request),
    ///     Some(UpgradeType::SSE)
    /// );
    ///
    /// // Regular HTTP request
    /// assert_eq!(UpgradeDetector::detect(&http_request), None);
    /// ```
    pub fn detect(req: &Request) -> Option<UpgradeType> {
        // Check for WebSocket upgrade first (explicit upgrade)
        if Self::is_websocket(req) {
            return Some(UpgradeType::WebSocket);
        }

        // Check for SSE (content negotiation)
        if Self::is_sse(req) {
            return Some(UpgradeType::SSE);
        }

        None
    }

    /// Check if request is a WebSocket upgrade
    ///
    /// WebSocket upgrade requires:
    /// - `Upgrade: websocket` header (case-insensitive)
    /// - `Sec-WebSocket-Key` header (must be present)
    ///
    /// # RFC 6455 (WebSocket Protocol)
    ///
    /// The WebSocket handshake requires specific headers:
    /// - `Upgrade: websocket` - Indicates protocol upgrade
    /// - `Connection: Upgrade` - Indicates connection upgrade (optional check)
    /// - `Sec-WebSocket-Key` - Client nonce for handshake
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::upgrade::UpgradeDetector;
    ///
    /// // Valid WebSocket request
    /// assert!(UpgradeDetector::is_websocket(&ws_request));
    ///
    /// // Regular HTTP request
    /// assert!(!UpgradeDetector::is_websocket(&http_request));
    /// ```
    pub fn is_websocket(req: &Request) -> bool {
        // Check for "Upgrade: websocket" header (case-insensitive)
        let has_upgrade = req
            .headers()
            .get("upgrade")
            .map(|v| v.to_lowercase() == "websocket")
            .unwrap_or(false);

        // Check for "Sec-WebSocket-Key" header (required for handshake)
        let has_key = req.headers().get("sec-websocket-key").is_some();

        has_upgrade && has_key
    }

    /// Check if request is a Server-Sent Events (SSE) request
    ///
    /// SSE detection requires:
    /// - `Accept: text/event-stream` header (can be part of Accept list)
    ///
    /// # SSE Protocol
    ///
    /// SSE uses content negotiation via the Accept header.
    /// The client indicates it can accept `text/event-stream` responses.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::upgrade::UpgradeDetector;
    ///
    /// // Valid SSE request
    /// assert!(UpgradeDetector::is_sse(&sse_request));
    ///
    /// // Regular HTTP request
    /// assert!(!UpgradeDetector::is_sse(&http_request));
    /// ```
    pub fn is_sse(req: &Request) -> bool {
        // Check for "Accept: text/event-stream" header
        // Can be part of a comma-separated list (e.g., "text/event-stream, text/html")
        req.headers()
            .get("accept")
            .map(|v| v.contains("text/event-stream"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Headers;
    use bytes::Bytes;
    use http::{Method, Uri};
    use std::collections::HashMap;

    fn create_request_with_headers(headers: Headers) -> Request {
        Request::new(
            Method::GET,
            Uri::from_static("http://localhost/test"),
            HashMap::new(), // Empty query params
            headers,
            Bytes::new(),
        )
    }

    #[test]
    fn test_detect_websocket() {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket");
        headers.set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==");

        let req = create_request_with_headers(headers);
        assert_eq!(UpgradeDetector::detect(&req), Some(UpgradeType::WebSocket));
    }

    #[test]
    fn test_detect_sse() {
        let mut headers = Headers::new();
        headers.set("accept", "text/event-stream");

        let req = create_request_with_headers(headers);
        assert_eq!(UpgradeDetector::detect(&req), Some(UpgradeType::SSE));
    }

    #[test]
    fn test_detect_none() {
        let headers = Headers::new();
        let req = create_request_with_headers(headers);
        assert_eq!(UpgradeDetector::detect(&req), None);
    }

    #[test]
    fn test_is_websocket_valid() {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket");
        headers.set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==");

        let req = create_request_with_headers(headers);
        assert!(UpgradeDetector::is_websocket(&req));
    }

    #[test]
    fn test_is_websocket_case_insensitive() {
        let mut headers = Headers::new();
        headers.set("upgrade", "WebSocket");
        headers.set("sec-websocket-key", "key");

        let req = create_request_with_headers(headers);
        assert!(UpgradeDetector::is_websocket(&req));
    }

    #[test]
    fn test_is_websocket_missing_key() {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket");

        let req = create_request_with_headers(headers);
        assert!(!UpgradeDetector::is_websocket(&req));
    }

    #[test]
    fn test_is_websocket_missing_upgrade() {
        let mut headers = Headers::new();
        headers.set("sec-websocket-key", "key");

        let req = create_request_with_headers(headers);
        assert!(!UpgradeDetector::is_websocket(&req));
    }

    #[test]
    fn test_is_websocket_wrong_upgrade() {
        let mut headers = Headers::new();
        headers.set("upgrade", "http/2.0");
        headers.set("sec-websocket-key", "key");

        let req = create_request_with_headers(headers);
        assert!(!UpgradeDetector::is_websocket(&req));
    }

    #[test]
    fn test_is_sse_valid() {
        let mut headers = Headers::new();
        headers.set("accept", "text/event-stream");

        let req = create_request_with_headers(headers);
        assert!(UpgradeDetector::is_sse(&req));
    }

    #[test]
    fn test_is_sse_in_accept_list() {
        let mut headers = Headers::new();
        headers.set("accept", "text/html, text/event-stream, application/json");

        let req = create_request_with_headers(headers);
        assert!(UpgradeDetector::is_sse(&req));
    }

    #[test]
    fn test_is_sse_missing_accept() {
        let headers = Headers::new();
        let req = create_request_with_headers(headers);
        assert!(!UpgradeDetector::is_sse(&req));
    }

    #[test]
    fn test_is_sse_wrong_accept() {
        let mut headers = Headers::new();
        headers.set("accept", "application/json");

        let req = create_request_with_headers(headers);
        assert!(!UpgradeDetector::is_sse(&req));
    }

    #[test]
    fn test_websocket_priority_over_sse() {
        // If both WebSocket and SSE headers are present, WebSocket takes priority
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket");
        headers.set("sec-websocket-key", "key");
        headers.set("accept", "text/event-stream");

        let req = create_request_with_headers(headers);
        assert_eq!(UpgradeDetector::detect(&req), Some(UpgradeType::WebSocket));
    }

    #[test]
    fn test_regular_http_request() {
        let mut headers = Headers::new();
        headers.set("accept", "text/html");
        headers.set("user-agent", "Mozilla/5.0");

        let req = create_request_with_headers(headers);
        assert_eq!(UpgradeDetector::detect(&req), None);
        assert!(!UpgradeDetector::is_websocket(&req));
        assert!(!UpgradeDetector::is_sse(&req));
    }
}
