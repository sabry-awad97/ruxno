//! Request domain model
//!
//! This module defines the `Request` type, which represents an HTTP request
//! in a protocol-agnostic way. The request uses Arc-wrapped internals for
//! cheap cloning and lazy body parsing with caching.
//!
//! # Design
//!
//! - **Arc-wrapped**: All data is wrapped in `Arc` for cheap cloning
//! - **Lazy parsing**: Body is parsed on-demand and cached
//! - **Immutable**: Request is immutable after construction (params set via `with_params`)
//! - **HTTP-aware**: Uses `http::Uri` for proper URI parsing and `http::HeaderMap` for headers
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::Request;
//!
//! async fn handler(req: Request) {
//!     // Access request properties
//!     let method = req.method();
//!     let path = req.path();
//!     
//!     // Get path parameters
//!     let id = req.param("id").unwrap();
//!     
//!     // Get query parameters
//!     if let Some(page) = req.query("page") {
//!         println!("Page: {}", page);
//!     }
//!     
//!     // Parse body (lazy, cached)
//!     let body: MyStruct = req.json().await.unwrap();
//! }
//! ```

use crate::core::{CoreError, Method};
use crate::http::Headers;
use crate::routing::Params;
use bytes::Bytes;
use http::Uri;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Request domain model
///
/// Represents an HTTP request in a protocol-agnostic way. Uses Arc-wrapped
/// internals for cheap cloning and lazy body parsing with caching.
///
/// # Cloning
///
/// Cloning a `Request` is cheap (just increments Arc reference counts).
/// This allows passing requests through middleware chains efficiently.
///
/// # Body Parsing
///
/// Body parsing is lazy and cached. The first call to `json()`, `text()`,
/// or `form()` parses the body and caches the result. Subsequent calls
/// return the cached value.
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::Request;
///
/// async fn handler(req: Request) {
///     // Cheap clone
///     let req2 = req.clone();
///     
///     // Access properties
///     println!("Method: {}", req.method());
///     println!("Path: {}", req.path());
///     
///     // Get parameters
///     if let Ok(id) = req.param("id") {
///         println!("ID: {}", id);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Request {
    /// Inner data wrapped in Arc for cheap cloning
    inner: Arc<RequestInner>,
}

/// Inner request data
struct RequestInner {
    /// HTTP method
    method: Method,

    /// Request URI (parsed HTTP URI with path, query, etc.)
    uri: Uri,

    /// Query parameters (parsed from URI)
    query: HashMap<String, String>,

    /// Headers (HTTP-aware, case-insensitive, multi-value support)
    headers: Headers,

    /// Raw body bytes
    body: Bytes,

    /// Path parameters (set by router, immutable after construction)
    params: Params,

    /// Cached parsed body (lazy initialization)
    #[allow(dead_code)]
    body_cache: RwLock<BodyCache>,
}

/// Cached parsed body variants
#[derive(Default)]
struct BodyCache {
    /// Cached JSON string (for re-parsing into different types)
    json: Option<String>,

    /// Cached text
    text: Option<String>,

    /// Cached form data
    form: Option<HashMap<String, String>>,
}

impl Request {
    /// Create a new request
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method
    /// - `uri`: HTTP URI (includes path and query string)
    /// - `query`: Parsed query parameters
    /// - `headers`: Request headers (Headers wrapper)
    /// - `body`: Raw body bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let uri = "/api/users?page=1".parse().unwrap();
    /// let req = Request::new(
    ///     Method::GET,
    ///     uri,
    ///     HashMap::from([("page".to_string(), "1".to_string())]),
    ///     Headers::new(),
    ///     Bytes::new(),
    /// );
    /// ```
    pub fn new(
        method: Method,
        uri: Uri,
        query: HashMap<String, String>,
        headers: Headers,
        body: Bytes,
    ) -> Self {
        Self {
            inner: Arc::new(RequestInner {
                method,
                uri,
                query,
                headers,
                body,
                params: Params::new(),
                body_cache: RwLock::new(BodyCache::default()),
            }),
        }
    }

    /// Create a request with path parameters (used by router)
    ///
    /// This is the preferred way for the router to inject path parameters
    /// after route matching. It avoids the need for mutable state or locks.
    ///
    /// # Arguments
    ///
    /// - `params`: Path parameters extracted from route
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Router extracts params from route pattern
    /// let params = HashMap::from([("id".to_string(), "123".to_string())]);
    /// let req = base_req.with_params(params);
    /// ```
    pub fn with_params(self, params: Params) -> Self {
        // Create new RequestInner with updated params
        // This avoids Arc::make_mut which requires Clone on RequestInner
        Self {
            inner: Arc::new(RequestInner {
                method: self.inner.method.clone(),
                uri: self.inner.uri.clone(),
                query: self.inner.query.clone(),
                headers: self.inner.headers.clone(),
                body: self.inner.body.clone(),
                params,
                body_cache: RwLock::new(BodyCache::default()),
            }),
        }
    }

    /// Get HTTP method
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(req.method(), &Method::GET);
    /// ```
    pub fn method(&self) -> &Method {
        &self.inner.method
    }

    /// Get full URI (including query string)
    ///
    /// Returns the parsed HTTP URI which includes path, query, and optionally
    /// scheme, authority, etc.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(req.uri().path(), "/api/users");
    /// assert_eq!(req.uri().query(), Some("page=1"));
    /// ```
    pub fn uri(&self) -> &Uri {
        &self.inner.uri
    }

    /// Get request path (without query string)
    ///
    /// This is a convenience method that extracts the path from the URI.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(req.path(), "/api/users");
    /// ```
    pub fn path(&self) -> &str {
        self.inner.uri.path()
    }

    /// Get a query parameter by key
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some(page) = req.query("page") {
    ///     println!("Page: {}", page);
    /// }
    /// ```
    pub fn query(&self, key: &str) -> Option<&str> {
        self.inner.query.get(key).map(|s| s.as_str())
    }

    /// Get all query parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for (key, value) in req.query_all() {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn query_all(&self) -> &HashMap<String, String> {
        &self.inner.query
    }

    /// Get a header by key (case-insensitive)
    ///
    /// Returns the first value if multiple values exist for the header.
    /// Use `header_all()` to get all values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some(content_type) = req.header("content-type") {
    ///     println!("Content-Type: {}", content_type);
    /// }
    /// ```
    pub fn header(&self, key: &str) -> Option<&str> {
        self.inner.headers.get(key)
    }

    /// Get all values for a header (case-insensitive)
    ///
    /// Returns an iterator over all values for the given header key.
    /// Useful for headers that can have multiple values (e.g., Set-Cookie).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for cookie in req.header_all("set-cookie") {
    ///     println!("Cookie: {}", cookie);
    /// }
    /// ```
    pub fn header_all(&self, key: &str) -> Vec<&str> {
        self.inner.headers.get_all(key).collect()
    }

    /// Get all headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = req.headers();
    /// for (key, value) in headers.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    pub fn headers(&self) -> &Headers {
        &self.inner.headers
    }

    /// Get a path parameter by key
    ///
    /// Path parameters are extracted from the route pattern by the router.
    /// For example, a route `/users/:id` would extract `id` as a parameter.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::MissingParameter` if the parameter doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Route: /users/:id
    /// // Request: /users/123
    /// let id = req.param("id")?; // "123"
    /// ```
    pub fn param(&self, key: &str) -> Result<&str, CoreError> {
        self.inner
            .params
            .get(key)
            .ok_or_else(|| CoreError::missing_parameter(key))
    }

    /// Get all path parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = req.params();
    /// for (key, value) in params {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn params(&self) -> &Params {
        &self.inner.params
    }

    /// Get raw body bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bytes = req.body();
    /// println!("Body size: {} bytes", bytes.len());
    /// ```
    pub fn body(&self) -> &Bytes {
        &self.inner.body
    }

    /// Parse body as JSON
    ///
    /// Uses the `JsonParser` with default size limits (2MB).
    /// The body is parsed from raw bytes for optimal performance.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::BodyParseError` if the body is not valid JSON.
    /// Returns `CoreError::BadRequest` if body exceeds size limit.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user: User = req.json().await?;
    /// println!("User: {}", user.name);
    /// ```
    pub async fn json<T>(&self) -> Result<T, CoreError>
    where
        T: serde::de::DeserializeOwned,
    {
        // Use JsonParser for consistent parsing with size limits
        crate::body::JsonParser::parse_as(&self.inner.body).await
    }

    /// Parse body as plain text
    ///
    /// The body is parsed lazily and the result is cached.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::BodyParseError` if the body is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let text = req.text().await?;
    /// println!("Body: {}", text);
    /// ```
    pub async fn text(&self) -> Result<String, CoreError> {
        // Check cache first
        {
            let cache = self.inner.body_cache.read().await;
            if let Some(text) = &cache.text {
                return Ok(text.clone());
            }
        }

        // Parse and cache
        let text = String::from_utf8(self.inner.body.to_vec())
            .map_err(|e| CoreError::body_parse_error(format!("Invalid UTF-8: {}", e)))?;

        // Cache the result
        self.inner.body_cache.write().await.text = Some(text.clone());

        Ok(text)
    }

    /// Parse body as URL-encoded form data
    ///
    /// The body is parsed from cached text. This ensures consistent behavior
    /// with other parsing methods.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::BodyParseError` if the body is not valid form data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let form = req.form().await?;
    /// if let Some(email) = form.get("email") {
    ///     println!("Email: {}", email);
    /// }
    /// ```
    pub async fn form(&self) -> Result<HashMap<String, String>, CoreError> {
        // Parse form data from cached text
        let text = self.text().await?;
        serde_urlencoded::from_str(&text)
            .map_err(|e| CoreError::body_parse_error(format!("Invalid form data: {}", e)))
    }

    /// Get body as raw bytes (alias for `body()`)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bytes = req.bytes();
    /// ```
    pub fn bytes(&self) -> &Bytes {
        self.body()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Method;

    fn create_test_request() -> Request {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();
        headers.set("authorization", "Bearer token123").unwrap();

        Request::new(
            Method::GET,
            "/api/users?page=1&limit=10".parse().unwrap(),
            HashMap::from([
                ("page".to_string(), "1".to_string()),
                ("limit".to_string(), "10".to_string()),
            ]),
            headers,
            Bytes::from(r#"{"name":"John","email":"john@example.com"}"#),
        )
    }

    #[test]
    fn test_request_method() {
        let req = create_test_request();
        assert_eq!(req.method(), &Method::GET);
    }

    #[test]
    fn test_request_uri() {
        let req = create_test_request();
        assert_eq!(req.uri().path(), "/api/users");
        assert_eq!(req.uri().query(), Some("page=1&limit=10"));
    }

    #[test]
    fn test_request_path() {
        let req = create_test_request();
        assert_eq!(req.path(), "/api/users");
    }

    #[test]
    fn test_request_query() {
        let req = create_test_request();
        assert_eq!(req.query("page"), Some("1"));
        assert_eq!(req.query("limit"), Some("10"));
        assert_eq!(req.query("nonexistent"), None);
    }

    #[test]
    fn test_request_query_all() {
        let req = create_test_request();
        let query = req.query_all();
        assert_eq!(query.len(), 2);
        assert_eq!(query.get("page"), Some(&"1".to_string()));
        assert_eq!(query.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_request_header() {
        let req = create_test_request();
        assert_eq!(req.header("content-type"), Some("application/json"));
        assert_eq!(req.header("Content-Type"), Some("application/json")); // Case-insensitive
        assert_eq!(req.header("authorization"), Some("Bearer token123"));
        assert_eq!(req.header("nonexistent"), None);
    }

    #[test]
    fn test_request_header_all() {
        let mut headers = Headers::new();
        headers.append("set-cookie", "cookie1=value1").unwrap();
        headers.append("set-cookie", "cookie2=value2").unwrap();

        let req = Request::new(
            Method::GET,
            "/".parse().unwrap(),
            HashMap::new(),
            headers,
            Bytes::new(),
        );

        let cookies = req.header_all("set-cookie");
        assert_eq!(cookies.len(), 2);
        assert!(cookies.contains(&"cookie1=value1"));
        assert!(cookies.contains(&"cookie2=value2"));
    }

    #[test]
    fn test_request_headers() {
        let req = create_test_request();
        let headers = req.headers();
        assert_eq!(headers.len(), 2);
        assert!(headers.contains("content-type"));
        assert!(headers.contains("authorization"));
    }

    #[test]
    fn test_request_param() {
        let req = create_test_request();

        // Set params using with_params
        let req = req.with_params(Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]));

        assert_eq!(req.param("id").unwrap(), "123");
        assert_eq!(req.param("name").unwrap(), "john");
        assert!(req.param("nonexistent").is_err());
    }

    #[test]
    fn test_request_params() {
        let req = create_test_request();

        let req = req.with_params(Params::from(vec![("id".to_string(), "123".to_string())]));

        let params = req.params();
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("id"), Some("123"));
    }

    #[test]
    fn test_request_with_params_immutable() {
        let req1 = create_test_request();
        let req2 = req1.clone();

        // Set params on req2
        let req2 = req2.with_params(Params::from(vec![("id".to_string(), "123".to_string())]));

        // req1 should not have params (immutable)
        assert!(req1.param("id").is_err());
        assert_eq!(req2.param("id").unwrap(), "123");
    }

    #[test]
    fn test_request_body() {
        let req = create_test_request();
        let body = req.body();
        assert!(!body.is_empty());
        assert_eq!(
            body,
            &Bytes::from(r#"{"name":"John","email":"john@example.com"}"#)
        );
    }

    #[test]
    fn test_request_bytes() {
        let req = create_test_request();
        let bytes = req.bytes();
        assert_eq!(bytes, req.body());
    }

    #[tokio::test]
    async fn test_request_json() {
        use serde::Deserialize;

        #[derive(Deserialize, Debug, PartialEq)]
        struct User {
            name: String,
            email: String,
        }

        let req = create_test_request();
        let user: User = req.json().await.unwrap();

        assert_eq!(user.name, "John");
        assert_eq!(user.email, "john@example.com");
    }

    #[tokio::test]
    async fn test_request_json_invalid() {
        let req = Request::new(
            Method::POST,
            "/api/users".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::from("invalid json"),
        );

        let result: Result<serde_json::Value, _> = req.json().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_text() {
        let req = Request::new(
            Method::POST,
            "/api/echo".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::from("Hello, World!"),
        );

        let text = req.text().await.unwrap();
        assert_eq!(text, "Hello, World!");

        // Test caching - second call should return cached value
        let text2 = req.text().await.unwrap();
        assert_eq!(text2, "Hello, World!");
    }

    #[tokio::test]
    async fn test_request_form() {
        let req = Request::new(
            Method::POST,
            "/api/login".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::from("email=john%40example.com&password=secret123"),
        );

        let form = req.form().await.unwrap();
        assert_eq!(form.get("email"), Some(&"john@example.com".to_string()));
        assert_eq!(form.get("password"), Some(&"secret123".to_string()));

        // Test caching
        let form2 = req.form().await.unwrap();
        assert_eq!(form2, form);
    }

    #[test]
    fn test_request_clone() {
        let req = create_test_request();
        let req2 = req.clone();

        // Both should have same data
        assert_eq!(req.method(), req2.method());
        assert_eq!(req.path(), req2.path());
        assert_eq!(req.uri(), req2.uri());

        // Arc reference count should be 2
        assert_eq!(Arc::strong_count(&req.inner), 2);
    }
}
