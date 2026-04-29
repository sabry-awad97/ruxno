//! HTTP Sniffer Utility
//!
//! Logs detailed HTTP request information including method, version, URL, headers,
//! and query parameters.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::prelude::*;
//! use crate::middleware::http_sniffer;
//!
//! let app = App::new()
//!     .get("/", |ctx: Context| async move {
//!         // Sniff the request
//!         http_sniffer::sniff_request(ctx.req());
//!         ctx.text("Hello, World!")
//!     });
//! ```

use async_trait::async_trait;
use chrono::Utc;
use ruxno::prelude::*;

/// HTTP Sniffer middleware for detailed request logging
///
/// Captures and logs comprehensive HTTP request information including:
/// - Timestamp (ISO 8601 format)
/// - HTTP method and version
/// - Request URL and parsed components
/// - All headers with enumerated output
/// - Query parameters
/// - Connection information
///
/// # Design
///
/// This middleware is designed for debugging and monitoring purposes.
/// It provides detailed visibility into incoming HTTP requests without
/// affecting the request/response flow.
pub struct HttpSniffer {
    /// Whether to log request bodies (disabled by default for security)
    log_body: bool,
    /// Maximum body size to log (in bytes)
    max_body_size: usize,
}

impl HttpSniffer {
    /// Create a new HTTP sniffer with default settings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let sniffer = HttpSniffer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            log_body: false,
            max_body_size: 1024, // 1KB default
        }
    }

    /// Enable body logging with size limit
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum body size to log in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let sniffer = HttpSniffer::new().with_body_logging(512);
    /// ```
    pub fn with_body_logging(mut self, max_size: usize) -> Self {
        self.log_body = true;
        self.max_body_size = max_size;
        self
    }

    /// Get current timestamp in ISO 8601 format
    fn timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    /// Convert request to detailed string representation
    fn req_to_string(&self, req: &Request) -> String {
        let mut output = String::new();

        // Request line
        output.push_str(&format!(
            "request {} {:?} {}\n",
            req.method().as_str(),
            req.version(),
            req.path()
        ));

        // Parsed URL information
        let uri = req.uri();
        let parsed_url = serde_json::json!({
            "protocol": uri.scheme_str().unwrap_or(""),
            "host": uri.host().unwrap_or(""),
            "port": uri.port_u16(),
            "pathname": uri.path(),
            "search": uri.query().map(|q| format!("?{}", q)).unwrap_or_default(),
            "query": req.query_all()
        });

        output.push_str(&format!(
            "{}\n",
            serde_json::to_string_pretty(&parsed_url).unwrap_or_default()
        ));

        // Headers with enumeration
        let headers = req.headers();

        for (header_count, (name, value)) in headers.iter().enumerate() {
            output.push_str(&format!("{} {}: {}\n", header_count, name, value));
        }

        // Body information (if enabled)
        if self.log_body {
            let body = req.body();
            if !body.is_empty() {
                let body_size = body.len();
                if body_size <= self.max_body_size {
                    if let Ok(body_str) = std::str::from_utf8(body) {
                        output.push_str(&format!("Body ({} bytes):\n{}\n", body_size, body_str));
                    } else {
                        output.push_str(&format!("Body ({} bytes): [binary data]\n", body_size));
                    }
                } else {
                    output.push_str(&format!(
                        "Body ({} bytes): [truncated - exceeds {} byte limit]\n",
                        body_size, self.max_body_size
                    ));
                }
            }
        }

        output
    }

    /// Log request event
    fn log_request(&self, req: &Request) {
        let timestamp = Self::timestamp();
        println!("{} request", timestamp);
        println!("{} {}", timestamp, self.req_to_string(req));
    }

    /// Log server events (for demonstration - would be implemented at server level)
    #[allow(dead_code)]
    fn log_server_event(event: &str, details: Option<&str>) {
        let timestamp = Self::timestamp();
        match details {
            Some(detail) => println!("{} {} {}", timestamp, event, detail),
            None => println!("{} {}", timestamp, event),
        }
    }
}

impl Default for HttpSniffer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> Middleware<E> for HttpSniffer
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, RuxnoError> {
        // Log the incoming request
        self.log_request(&ctx.req);

        // Continue to next middleware/handler
        let response = next.run(ctx).await?;

        // Could also log response information here if needed
        // self.log_response(&response);

        Ok(response)
    }
}

/// Extension trait for easy integration with App
pub trait HttpSnifferExt<E = ()> {
    /// Add HTTP sniffer middleware to all routes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut app = App::new();
    /// app.with_http_sniffer()
    ///     .get("/", handler);
    /// ```
    fn with_http_sniffer(&mut self) -> &mut Self;

    /// Add HTTP sniffer middleware with body logging
    ///
    /// # Arguments
    ///
    /// * `max_body_size` - Maximum body size to log in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut app = App::new();
    /// app.with_http_sniffer_body(512)
    ///     .get("/", handler);
    /// ```
    fn with_http_sniffer_body(&mut self, max_body_size: usize) -> &mut Self;
}

impl<E> HttpSnifferExt<E> for App<E>
where
    E: Send + Sync + 'static,
{
    fn with_http_sniffer(&mut self) -> &mut Self {
        self.r#use(HttpSniffer::new())
    }

    fn with_http_sniffer_body(&mut self, max_body_size: usize) -> &mut Self {
        self.r#use(HttpSniffer::new().with_body_logging(max_body_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::collections::HashMap;

    fn create_test_request() -> Request {
        let mut headers = ruxno::http::Headers::new();
        headers.set("user-agent", "Mozilla/5.0").unwrap();
        headers.set("accept", "text/html,application/json").unwrap();
        headers.set("host", "localhost:3000").unwrap();

        Request::new(
            Method::GET,
            "http://localhost:3000/api/users?page=1&limit=10"
                .parse()
                .unwrap(),
            http::Version::HTTP_11,
            HashMap::from([
                ("page".to_string(), "1".to_string()),
                ("limit".to_string(), "10".to_string()),
            ]),
            headers,
            Bytes::from(r#"{"test": "data"}"#),
        )
    }

    #[test]
    fn test_http_sniffer_new() {
        let sniffer = HttpSniffer::new();
        assert!(!sniffer.log_body);
        assert_eq!(sniffer.max_body_size, 1024);
    }

    #[test]
    fn test_http_sniffer_with_body_logging() {
        let sniffer = HttpSniffer::new().with_body_logging(512);
        assert!(sniffer.log_body);
        assert_eq!(sniffer.max_body_size, 512);
    }

    #[test]
    fn test_req_to_string() {
        let sniffer = HttpSniffer::new();
        let req = create_test_request();
        let output = sniffer.req_to_string(&req);

        // Check that output contains expected elements
        assert!(output.contains("request GET"));
        assert!(output.contains("HTTP_11"));
        assert!(output.contains("/api/users"));
        assert!(output.contains("user-agent: Mozilla/5.0"));
        assert!(output.contains("accept: text/html,application/json"));
        assert!(output.contains("host: localhost:3000"));

        // Should contain parsed URL JSON
        assert!(output.contains("\"pathname\": \"/api/users\""));
        assert!(output.contains("\"page\": \"1\""));
        assert!(output.contains("\"limit\": \"10\""));
    }

    #[test]
    fn test_req_to_string_with_body() {
        let sniffer = HttpSniffer::new().with_body_logging(1024);
        let req = create_test_request();
        let output = sniffer.req_to_string(&req);

        // Should include body information
        assert!(output.contains("Body ("));
        assert!(output.contains("bytes):"));
        assert!(output.contains(r#"{"test": "data"}"#));
    }

    #[test]
    fn test_req_to_string_body_size_limit() {
        let sniffer = HttpSniffer::new().with_body_logging(5); // Very small limit
        let req = create_test_request();
        let output = sniffer.req_to_string(&req);

        // Should indicate truncation
        assert!(output.contains("truncated - exceeds"));
        assert!(output.contains("5 byte limit"));
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = HttpSniffer::timestamp();

        // Should be in ISO 8601 format (basic check)
        assert!(timestamp.contains("T"));
        assert!(timestamp.contains("Z") || timestamp.contains("+"));
        assert!(timestamp.len() >= 19); // Minimum length for ISO format
    }

    #[tokio::test]
    async fn test_middleware_integration() {
        let sniffer = HttpSniffer::new();
        let req = create_test_request();
        let ctx = Context::new(req, std::sync::Arc::new(()));

        // Create a simple next handler
        let next =
            Next::from(|_ctx: Context<()>| async move { Ok(Response::text("Hello, World!")) });

        // Process through middleware
        let result = sniffer.process(ctx, next).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should pass through response unchanged
        // Note: We can't easily test the console output in unit tests
        // but we can verify the middleware doesn't interfere with the response
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_app_extension_trait() {
        let mut app = App::new();
        app.with_http_sniffer().get(
            "/test",
            |ctx: Context<()>| async move { Ok(ctx.text("Test")) },
        );

        // App should be created successfully with sniffer middleware
        // This is mainly a compilation test
        drop(app);
    }

    #[test]
    fn test_app_extension_trait_with_body() {
        let mut app = App::new();
        app.with_http_sniffer_body(256)
            .get(
                "/test",
                |ctx: Context<()>| async move { Ok(ctx.text("Test")) },
            );

        // App should be created successfully with body logging sniffer
        drop(app);
    }
}
