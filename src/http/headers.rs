//! HTTP header utilities
//!
//! This module provides a convenient wrapper around `http::HeaderMap` with
//! ergonomic methods for common header operations and typed accessors for
//! standard HTTP headers.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::http::Headers;
//!
//! let mut headers = Headers::new();
//! headers.set("content-type", "application/json");
//! headers.set("authorization", "Bearer token123");
//!
//! // Typed accessors
//! let content_type = headers.content_type();
//! let auth = headers.authorization();
//! ```

use http::HeaderMap;

/// HTTP headers wrapper
///
/// Wraps `http::HeaderMap` and provides convenient methods for working with
/// HTTP headers. Supports case-insensitive header names and multi-value headers.
///
/// # Examples
///
/// ```rust,ignore
/// let mut headers = Headers::new();
///
/// // Set headers
/// headers.set("content-type", "application/json");
/// headers.set("x-custom-header", "custom-value");
///
/// // Get headers
/// assert_eq!(headers.get("content-type"), Some("application/json"));
/// assert_eq!(headers.get("Content-Type"), Some("application/json")); // Case-insensitive
///
/// // Typed accessors
/// let content_type = headers.content_type();
/// ```
#[derive(Debug, Clone)]
pub struct Headers {
    inner: HeaderMap,
}

impl Headers {
    /// Create new empty headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = Headers::new();
    /// assert!(headers.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HeaderMap::new(),
        }
    }

    /// Convert to `http::HeaderMap`
    ///
    /// Consumes the `Headers` and returns the inner `HeaderMap`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = Headers::new();
    /// let header_map = headers.into_header_map();
    /// ```
    pub fn into_header_map(self) -> HeaderMap {
        self.inner
    }

    /// Get a reference to the inner `HeaderMap`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = Headers::new();
    /// let header_map = headers.as_header_map();
    /// ```
    pub fn as_header_map(&self) -> &HeaderMap {
        &self.inner
    }

    /// Get a mutable reference to the inner `HeaderMap`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// let header_map = headers.as_header_map_mut();
    /// ```
    pub fn as_header_map_mut(&mut self) -> &mut HeaderMap {
        &mut self.inner
    }

    /// Get header value by key (case-insensitive)
    ///
    /// Returns the first value if multiple values exist for the header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    ///
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// assert_eq!(headers.get("Content-Type"), Some("application/json")); // Case-insensitive
    /// ```
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).and_then(|v| v.to_str().ok())
    }

    /// Set header value (case-insensitive)
    ///
    /// Replaces any existing values for the header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    /// ```
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), HeaderError> {
        let header_name = key
            .parse::<http::HeaderName>()
            .map_err(|e| HeaderError::InvalidName(e.to_string()))?;

        let header_value = value
            .parse::<http::HeaderValue>()
            .map_err(|e| HeaderError::InvalidValue(e.to_string()))?;

        self.inner.insert(header_name, header_value);
        Ok(())
    }

    /// Append header value (case-insensitive)
    ///
    /// Adds a new value without removing existing values.
    /// Useful for headers that can have multiple values (e.g., Set-Cookie).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.append("set-cookie", "cookie1=value1");
    /// headers.append("set-cookie", "cookie2=value2");
    ///
    /// let cookies: Vec<_> = headers.get_all("set-cookie").collect();
    /// assert_eq!(cookies.len(), 2);
    /// ```
    pub fn append(&mut self, key: &str, value: &str) -> Result<(), HeaderError> {
        let header_name = key
            .parse::<http::HeaderName>()
            .map_err(|e| HeaderError::InvalidName(e.to_string()))?;

        let header_value = value
            .parse::<http::HeaderValue>()
            .map_err(|e| HeaderError::InvalidValue(e.to_string()))?;

        self.inner.append(header_name, header_value);
        Ok(())
    }

    /// Remove header by key (case-insensitive)
    ///
    /// Removes all values for the header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    /// headers.remove("content-type");
    ///
    /// assert!(headers.get("content-type").is_none());
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let header_name = key.parse::<http::HeaderName>().ok()?;
        self.inner
            .remove(header_name)
            .and_then(|v| v.to_str().ok().map(|s| s.to_string()))
    }

    /// Get all values for a header (case-insensitive)
    ///
    /// Returns an iterator over all values for the given header key.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.append("set-cookie", "cookie1=value1");
    /// headers.append("set-cookie", "cookie2=value2");
    ///
    /// let cookies: Vec<_> = headers.get_all("set-cookie").collect();
    /// assert_eq!(cookies.len(), 2);
    /// ```
    pub fn get_all(&self, key: &str) -> impl Iterator<Item = &str> {
        let header_name = key.parse::<http::HeaderName>().ok();
        header_name
            .map(|name| {
                self.inner
                    .get_all(name)
                    .iter()
                    .filter_map(|v| v.to_str().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
    }

    /// Check if header exists (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    ///
    /// assert!(headers.contains("content-type"));
    /// assert!(!headers.contains("authorization"));
    /// ```
    pub fn contains(&self, key: &str) -> bool {
        key.parse::<http::HeaderName>()
            .ok()
            .map(|name| self.inner.contains_key(name))
            .unwrap_or(false)
    }

    /// Get the number of headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    /// headers.set("authorization", "Bearer token");
    ///
    /// assert_eq!(headers.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if headers are empty
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let headers = Headers::new();
    /// assert!(headers.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    /// headers.clear();
    ///
    /// assert!(headers.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterate over all headers
    ///
    /// Returns an iterator over `(&str, &str)` pairs.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    /// headers.set("authorization", "Bearer token");
    ///
    /// for (key, value) in headers.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.as_str(), val)))
            .collect::<Vec<_>>()
            .into_iter()
    }

    // Typed accessors for common headers

    /// Get Content-Type header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-type", "application/json");
    ///
    /// assert_eq!(headers.content_type(), Some("application/json"));
    /// ```
    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type")
    }

    /// Set Content-Type header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set_content_type("application/json");
    /// ```
    pub fn set_content_type(&mut self, value: &str) -> Result<(), HeaderError> {
        self.set("content-type", value)
    }

    /// Get Authorization header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("authorization", "Bearer token123");
    ///
    /// assert_eq!(headers.authorization(), Some("Bearer token123"));
    /// ```
    pub fn authorization(&self) -> Option<&str> {
        self.get("authorization")
    }

    /// Set Authorization header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set_authorization("Bearer token123");
    /// ```
    pub fn set_authorization(&mut self, value: &str) -> Result<(), HeaderError> {
        self.set("authorization", value)
    }

    /// Get Content-Length header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("content-length", "1024");
    ///
    /// assert_eq!(headers.content_length(), Some(1024));
    /// ```
    pub fn content_length(&self) -> Option<u64> {
        self.get("content-length")
            .and_then(|v| v.parse::<u64>().ok())
    }

    /// Set Content-Length header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set_content_length(1024);
    /// ```
    pub fn set_content_length(&mut self, value: u64) -> Result<(), HeaderError> {
        self.set("content-length", &value.to_string())
    }

    /// Get User-Agent header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("user-agent", "Mozilla/5.0");
    ///
    /// assert_eq!(headers.user_agent(), Some("Mozilla/5.0"));
    /// ```
    pub fn user_agent(&self) -> Option<&str> {
        self.get("user-agent")
    }

    /// Set User-Agent header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set_user_agent("Mozilla/5.0");
    /// ```
    pub fn set_user_agent(&mut self, value: &str) -> Result<(), HeaderError> {
        self.set("user-agent", value)
    }

    /// Get Accept header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set("accept", "application/json");
    ///
    /// assert_eq!(headers.accept(), Some("application/json"));
    /// ```
    pub fn accept(&self) -> Option<&str> {
        self.get("accept")
    }

    /// Set Accept header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut headers = Headers::new();
    /// headers.set_accept("application/json");
    /// ```
    pub fn set_accept(&mut self, value: &str) -> Result<(), HeaderError> {
        self.set("accept", value)
    }
}

/// Header error
///
/// Errors that can occur when working with headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// Invalid header name
    InvalidName(String),
    /// Invalid header value
    InvalidValue(String),
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderError::InvalidName(msg) => write!(f, "Invalid header name: {}", msg),
            HeaderError::InvalidValue(msg) => write!(f, "Invalid header value: {}", msg),
        }
    }
}

impl std::error::Error for HeaderError {}

// Conversion traits

impl From<HeaderMap> for Headers {
    /// Create headers from `http::HeaderMap`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let header_map = http::HeaderMap::new();
    /// let headers = Headers::from_header_map(header_map);
    /// ```
    fn from(map: HeaderMap) -> Self {
        Self { inner: map }
    }
}

impl From<Headers> for HeaderMap {
    fn from(headers: Headers) -> Self {
        headers.into_header_map()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_new() {
        let headers = Headers::new();
        assert!(headers.is_empty());
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn test_headers_set_and_get() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();
        headers.set("authorization", "Bearer token123").unwrap();

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("authorization"), Some("Bearer token123"));
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_headers_case_insensitive() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("Content-Type"), Some("application/json"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("application/json"));
    }

    #[test]
    fn test_headers_remove() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        let removed = headers.remove("content-type");
        assert_eq!(removed, Some("application/json".to_string()));
        assert!(headers.get("content-type").is_none());
    }

    #[test]
    fn test_headers_append() {
        let mut headers = Headers::new();
        headers.append("set-cookie", "cookie1=value1").unwrap();
        headers.append("set-cookie", "cookie2=value2").unwrap();

        let cookies: Vec<_> = headers.get_all("set-cookie").collect();
        assert_eq!(cookies.len(), 2);
        assert!(cookies.contains(&"cookie1=value1"));
        assert!(cookies.contains(&"cookie2=value2"));
    }

    #[test]
    fn test_headers_contains() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        assert!(headers.contains("content-type"));
        assert!(headers.contains("Content-Type")); // Case-insensitive
        assert!(!headers.contains("authorization"));
    }

    #[test]
    fn test_headers_clear() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();
        headers.set("authorization", "Bearer token").unwrap();

        assert_eq!(headers.len(), 2);

        headers.clear();
        assert!(headers.is_empty());
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn test_headers_iter() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();
        headers.set("authorization", "Bearer token").unwrap();

        let mut count = 0;
        for (key, value) in headers.iter() {
            count += 1;
            assert!(key == "content-type" || key == "authorization");
            assert!(value == "application/json" || value == "Bearer token");
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_headers_content_type() {
        let mut headers = Headers::new();
        headers.set_content_type("application/json").unwrap();

        assert_eq!(headers.content_type(), Some("application/json"));
    }

    #[test]
    fn test_headers_authorization() {
        let mut headers = Headers::new();
        headers.set_authorization("Bearer token123").unwrap();

        assert_eq!(headers.authorization(), Some("Bearer token123"));
    }

    #[test]
    fn test_headers_content_length() {
        let mut headers = Headers::new();
        headers.set_content_length(1024).unwrap();

        assert_eq!(headers.content_length(), Some(1024));
    }

    #[test]
    fn test_headers_user_agent() {
        let mut headers = Headers::new();
        headers.set_user_agent("Mozilla/5.0").unwrap();

        assert_eq!(headers.user_agent(), Some("Mozilla/5.0"));
    }

    #[test]
    fn test_headers_accept() {
        let mut headers = Headers::new();
        headers.set_accept("application/json").unwrap();

        assert_eq!(headers.accept(), Some("application/json"));
    }

    #[test]
    fn test_headers_into_header_map() {
        let mut headers = Headers::new();
        headers.set("content-type", "application/json").unwrap();

        let header_map = headers.into_header_map();
        assert_eq!(header_map.len(), 1);
    }

    #[test]
    fn test_headers_default() {
        let headers = Headers::default();
        assert!(headers.is_empty());
    }

    #[test]
    fn test_headers_clone() {
        let mut headers1 = Headers::new();
        headers1.set("content-type", "application/json").unwrap();

        let headers2 = headers1.clone();
        assert_eq!(headers2.content_type(), Some("application/json"));
    }

    #[test]
    fn test_header_error_display() {
        let error = HeaderError::InvalidName("test".to_string());
        assert_eq!(error.to_string(), "Invalid header name: test");

        let error = HeaderError::InvalidValue("test".to_string());
        assert_eq!(error.to_string(), "Invalid header value: test");
    }

    #[test]
    fn test_headers_set_overwrites() {
        let mut headers = Headers::new();
        headers.set("content-type", "text/plain").unwrap();
        headers.set("content-type", "application/json").unwrap();

        assert_eq!(headers.content_type(), Some("application/json"));
        assert_eq!(headers.len(), 1);
    }
}
