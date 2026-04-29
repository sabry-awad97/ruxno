//! CORS (Cross-Origin Resource Sharing) middleware
//!
//! Handles CORS preflight requests and adds appropriate headers to enable
//! cross-origin requests from web browsers.
//!
//! # Examples
//!
//! ## Simple Usage (Allow All Origins)
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::cors;
//!
//! let mut app = App::new();
//!
//! // Allow all origins (development only!)
//! app.r#use(cors());
//! ```
//!
//! ## Production Configuration
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::CorsMiddleware;
//!
//! let mut app = App::new();
//!
//! app.r#use(
//!     CorsMiddleware::new()
//!         .allow_origin("https://example.com")
//!         .allow_origin("https://app.example.com")
//!         .allow_methods(&["GET", "POST", "PUT", "DELETE"])
//!         .allow_headers(&["Content-Type", "Authorization"])
//!         .allow_credentials(true)
//!         .max_age(3600)
//! );
//! ```

use async_trait::async_trait;
use ruxno::core::{CoreError, Middleware, Next, StatusCode};
use ruxno::{Context, Response};
use std::collections::HashSet;

/// CORS middleware for handling cross-origin requests
///
/// Provides comprehensive CORS support including:
/// - Origin validation
/// - Preflight request handling
/// - Credential support
/// - Custom headers and methods
/// - Configurable max age
#[derive(Debug, Clone)]
pub struct CorsMiddleware {
    /// Allowed origins (None = allow all)
    allowed_origins: Option<HashSet<String>>,
    /// Allowed HTTP methods
    allowed_methods: HashSet<String>,
    /// Allowed headers
    allowed_headers: HashSet<String>,
    /// Whether to allow credentials
    allow_credentials: bool,
    /// Max age for preflight cache (in seconds)
    max_age: Option<u32>,
    /// Whether to expose all headers
    expose_all_headers: bool,
    /// Specific headers to expose
    exposed_headers: HashSet<String>,
}

impl CorsMiddleware {
    /// Create a new CORS middleware with restrictive defaults
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("https://example.com");
    /// ```
    pub fn new() -> Self {
        Self {
            allowed_origins: Some(HashSet::new()),
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: ["Content-Type", "Authorization", "Accept"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
            expose_all_headers: false,
            exposed_headers: HashSet::new(),
        }
    }

    /// Create a permissive CORS configuration (allow all origins)
    ///
    /// ⚠️ **Warning**: Only use in development! This allows any origin.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::permissive();
    /// ```
    pub fn permissive() -> Self {
        Self {
            allowed_origins: None, // Allow all origins
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: HashSet::new(), // Will allow any header
            allow_credentials: true,
            max_age: Some(86400),
            expose_all_headers: true,
            exposed_headers: HashSet::new(),
        }
    }

    /// Allow a specific origin
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("https://example.com")
    ///     .allow_origin("https://app.example.com");
    /// ```
    pub fn allow_origin(mut self, origin: &str) -> Self {
        if self.allowed_origins.is_none() {
            self.allowed_origins = Some(HashSet::new());
        }
        if let Some(ref mut origins) = self.allowed_origins {
            origins.insert(origin.to_string());
        }
        self
    }

    /// Allow multiple origins
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_origins(&["https://example.com", "https://app.example.com"]);
    /// ```
    pub fn allow_origins(mut self, origins: &[&str]) -> Self {
        for origin in origins {
            self = self.allow_origin(origin);
        }
        self
    }

    /// Allow all origins (⚠️ development only!)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new().allow_any_origin();
    /// ```
    pub fn allow_any_origin(mut self) -> Self {
        self.allowed_origins = None;
        self
    }

    /// Set allowed HTTP methods
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_methods(&["GET", "POST", "PUT"]);
    /// ```
    pub fn allow_methods(mut self, methods: &[&str]) -> Self {
        self.allowed_methods = methods.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add an allowed HTTP method
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_method("PATCH");
    /// ```
    pub fn allow_method(mut self, method: &str) -> Self {
        self.allowed_methods.insert(method.to_string());
        self
    }

    /// Set allowed headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_headers(&["Content-Type", "Authorization", "X-API-Key"]);
    /// ```
    pub fn allow_headers(mut self, headers: &[&str]) -> Self {
        self.allowed_headers = headers.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Add an allowed header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_header("X-Custom-Header");
    /// ```
    pub fn allow_header(mut self, header: &str) -> Self {
        self.allowed_headers.insert(header.to_lowercase());
        self
    }

    /// Allow any header (permissive mode)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new().allow_any_header();
    /// ```
    pub fn allow_any_header(mut self) -> Self {
        self.allowed_headers.clear();
        self
    }

    /// Enable or disable credentials
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .allow_credentials(true);
    /// ```
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set max age for preflight cache
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .max_age(3600); // 1 hour
    /// ```
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Expose all response headers to the client
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .expose_all_headers();
    /// ```
    pub fn expose_all_headers(mut self) -> Self {
        self.expose_all_headers = true;
        self
    }

    /// Expose specific headers to the client
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cors = CorsMiddleware::new()
    ///     .expose_headers(&["X-Total-Count", "X-Page-Count"]);
    /// ```
    pub fn expose_headers(mut self, headers: &[&str]) -> Self {
        self.exposed_headers = headers.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Check if origin is allowed
    fn is_origin_allowed(&self, origin: &str) -> bool {
        match &self.allowed_origins {
            None => true, // Allow all origins
            Some(origins) => origins.contains(origin) || origins.contains("*"),
        }
    }

    /// Get allowed origins header value
    fn get_allowed_origin(&self, request_origin: Option<&str>) -> String {
        match (&self.allowed_origins, request_origin) {
            (None, Some(origin)) => origin.to_string(), // Echo back the origin
            (None, None) => "*".to_string(),
            (Some(origins), Some(origin)) if self.is_origin_allowed(origin) => origin.to_string(),
            (Some(origins), _) if origins.contains("*") => "*".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Handle preflight OPTIONS request
    fn handle_preflight(&self, ctx: &Context<impl Send + Sync + 'static>) -> Response {
        let origin = ctx.req.header("origin");

        // Check if origin is allowed
        if let Some(origin) = origin {
            if !self.is_origin_allowed(origin) {
                return Response::new().with_status_code(StatusCode::FORBIDDEN);
            }
        }

        let mut response = Response::new().with_status_code(StatusCode::NO_CONTENT);

        // Add CORS headers
        if let Some(origin) = origin {
            response = response.with_header(
                "access-control-allow-origin",
                self.get_allowed_origin(Some(origin)),
            );
        }

        // Allow credentials
        if self.allow_credentials {
            response = response.with_header("access-control-allow-credentials", "true");
        }

        // Allow methods
        let methods: Vec<String> = self.allowed_methods.iter().cloned().collect();
        response = response.with_header("access-control-allow-methods", methods.join(", "));

        // Allow headers
        if self.allowed_headers.is_empty() {
            // If no specific headers configured, echo back requested headers
            if let Some(requested_headers) = ctx.req.header("access-control-request-headers") {
                response = response.with_header("access-control-allow-headers", requested_headers);
            }
        } else {
            let headers: Vec<String> = self.allowed_headers.iter().cloned().collect();
            response = response.with_header("access-control-allow-headers", headers.join(", "));
        }

        // Max age
        if let Some(max_age) = self.max_age {
            response = response.with_header("access-control-max-age", max_age.to_string());
        }

        response
    }

    /// Add CORS headers to actual response
    fn add_cors_headers(&self, mut response: Response, request_origin: Option<&str>) -> Response {
        // Add origin header
        response = response.with_header(
            "access-control-allow-origin",
            self.get_allowed_origin(request_origin),
        );

        // Allow credentials
        if self.allow_credentials {
            response = response.with_header("access-control-allow-credentials", "true");
        }

        // Expose headers
        if self.expose_all_headers {
            response = response.with_header("access-control-expose-headers", "*");
        } else if !self.exposed_headers.is_empty() {
            let headers: Vec<String> = self.exposed_headers.iter().cloned().collect();
            response = response.with_header("access-control-expose-headers", headers.join(", "));
        }

        response
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> Middleware<E> for CorsMiddleware
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        let origin = ctx.req.header("origin").map(|s| s.to_string());
        let method = ctx.req.method().as_str().to_string();

        // Handle preflight OPTIONS request
        if method == "OPTIONS" {
            return Ok(self.handle_preflight(&ctx));
        }

        // Process the actual request
        let response = next.run(ctx).await?;

        // Add CORS headers to the response
        Ok(self.add_cors_headers(response, origin.as_deref()))
    }
}

/// Create a permissive CORS middleware (allow all origins)
///
/// ⚠️ **Warning**: Only use in development! This allows any origin.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::App;
/// use ruxno_middleware::cors;
///
/// let mut app = App::new();
/// app.r#use(cors());
/// ```
pub fn cors() -> CorsMiddleware {
    CorsMiddleware::permissive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_new() {
        let cors = CorsMiddleware::new();
        assert!(cors.allowed_origins.is_some());
        assert!(!cors.allow_credentials);
        assert_eq!(cors.max_age, Some(86400));
    }

    #[test]
    fn test_cors_permissive() {
        let cors = CorsMiddleware::permissive();
        assert!(cors.allowed_origins.is_none());
        assert!(cors.allow_credentials);
        assert!(cors.expose_all_headers);
    }

    #[test]
    fn test_allow_origin() {
        let cors = CorsMiddleware::new()
            .allow_origin("https://example.com")
            .allow_origin("https://app.example.com");

        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://app.example.com"));
        assert!(!cors.is_origin_allowed("https://evil.com"));
    }

    #[test]
    fn test_allow_any_origin() {
        let cors = CorsMiddleware::new().allow_any_origin();
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://evil.com"));
    }

    #[test]
    fn test_builder_pattern() {
        let cors = CorsMiddleware::new()
            .allow_origin("https://example.com")
            .allow_methods(&["GET", "POST"])
            .allow_headers(&["Content-Type"])
            .allow_credentials(true)
            .max_age(3600);

        assert!(cors.allow_credentials);
        assert_eq!(cors.max_age, Some(3600));
        assert!(cors.allowed_methods.contains("GET"));
        assert!(cors.allowed_methods.contains("POST"));
        assert!(!cors.allowed_methods.contains("DELETE"));
    }

    #[test]
    fn test_get_allowed_origin() {
        let cors = CorsMiddleware::new().allow_origin("https://example.com");

        assert_eq!(
            cors.get_allowed_origin(Some("https://example.com")),
            "https://example.com"
        );
        assert_eq!(cors.get_allowed_origin(Some("https://evil.com")), "null");

        let permissive = CorsMiddleware::permissive();
        assert_eq!(
            permissive.get_allowed_origin(Some("https://anything.com")),
            "https://anything.com"
        );
    }

    #[test]
    fn test_cors_function() {
        let cors = cors();
        assert!(cors.allowed_origins.is_none());
        assert!(cors.allow_credentials);
    }
}
