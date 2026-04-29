//! Form body parser
//!
//! This module provides URL-encoded form parsing functionality with
//! size limits and comprehensive error handling.
//!
//! # Design
//!
//! - **URL-encoded**: Parses `application/x-www-form-urlencoded` data
//! - **Size limits**: Configurable max body size (default 1MB)
//! - **Type-safe**: Generic parsing via `parse_as<T>()`
//! - **Error handling**: Clear error messages for common issues
//! - **Flexible**: Parse to HashMap or specific types
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::FormParser;
//! use bytes::Bytes;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let body = Bytes::from("name=Alice&age=30");
//!
//! // Parse to specific type
//! let user: User = FormParser::parse_as(&body).await?;
//!
//! // Parse with size limit
//! let user: User = FormParser::with_limit(512).parse_as(&body).await?;
//! # Ok(())
//! # }
//! ```

use crate::body::BodyParser;
use crate::core::CoreError;
use async_trait::async_trait;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Default maximum body size for forms (1MB)
const DEFAULT_MAX_SIZE: usize = 1024 * 1024;

/// Form parser with configurable size limits
///
/// Parses URL-encoded form data (`application/x-www-form-urlencoded`)
/// with optional size validation. Provides both HashMap parsing and
/// type-specific parsing via `parse_as<T>()`.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::FormParser;
///
/// // Default parser (1MB limit)
/// let parser = FormParser::new();
///
/// // Custom size limit
/// let parser = FormParser::with_limit(512 * 1024); // 512KB
///
/// // Parse to specific type
/// let data: MyStruct = FormParser::parse_as(&body).await?;
/// ```
#[derive(Debug, Clone)]
pub struct FormParser {
    /// Maximum allowed body size in bytes
    max_size: usize,
}

impl FormParser {
    /// Create a new form parser with default size limit (1MB)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = FormParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    /// Create a form parser with custom size limit
    ///
    /// # Arguments
    ///
    /// - `max_size`: Maximum body size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = FormParser::with_limit(512 * 1024); // 512KB limit
    /// ```
    pub fn with_limit(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Get the configured size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = FormParser::with_limit(1024);
    /// assert_eq!(parser.max_size(), 1024);
    /// ```
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Parse form body into specific type (static method)
    ///
    /// Convenience method for parsing without creating a parser instance.
    /// Uses default size limit (1MB).
    ///
    /// # Type Parameters
    ///
    /// - `T`: Type to deserialize into (must implement `DeserializeOwned`)
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns the parsed value or an error.
    ///
    /// # Errors
    ///
    /// - `CoreError::BodyParseError` if form data is invalid
    /// - `CoreError::BadRequest` if body exceeds size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let user: User = FormParser::parse_as(&body).await?;
    /// ```
    pub async fn parse_as<T: DeserializeOwned>(body: &Bytes) -> Result<T, CoreError> {
        Self::new().parse_as_with_limit(body).await
    }

    /// Parse form body into specific type with size limit check
    ///
    /// # Type Parameters
    ///
    /// - `T`: Type to deserialize into
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns the parsed value or an error.
    ///
    /// # Errors
    ///
    /// - `CoreError::BodyParseError` if form data is invalid or not UTF-8
    /// - `CoreError::BadRequest` if body exceeds size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = FormParser::with_limit(1024);
    /// let user: User = parser.parse_as_with_limit(&body).await?;
    /// ```
    pub async fn parse_as_with_limit<T: DeserializeOwned>(
        &self,
        body: &Bytes,
    ) -> Result<T, CoreError> {
        // Check size limit
        if body.len() > self.max_size {
            return Err(CoreError::bad_request(format!(
                "Request body too large: {} bytes (max: {} bytes)",
                body.len(),
                self.max_size
            )));
        }

        // Convert to UTF-8 string
        let text = String::from_utf8(body.to_vec()).map_err(|e| {
            CoreError::body_parse_error(format!("Form data is not valid UTF-8: {}", e))
        })?;

        // Parse URL-encoded form data
        serde_urlencoded::from_str(&text).map_err(|e| {
            CoreError::body_parse_error(format!(
                "Invalid form data: {}",
                Self::format_form_error(&e)
            ))
        })
    }

    /// Format form parsing error for better user experience
    ///
    /// Extracts useful information from serde_urlencoded errors.
    fn format_form_error(error: &serde_urlencoded::de::Error) -> String {
        let error_str = error.to_string();

        // Provide more helpful error messages
        if error_str.contains("duplicate") {
            "Duplicate field in form data".to_string()
        } else if error_str.contains("missing") {
            "Missing required field in form data".to_string()
        } else if error_str.contains("invalid type") {
            "Invalid data type in form field".to_string()
        } else {
            error_str
        }
    }
}

impl Default for FormParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BodyParser for FormParser {
    type Output = HashMap<String, String>;

    /// Parse form body into HashMap<String, String>
    ///
    /// This implementation parses the body into a simple key-value map.
    /// For type-specific parsing, use `parse_as<T>()` instead.
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns a HashMap of form fields or an error.
    ///
    /// # Errors
    ///
    /// - `CoreError::BodyParseError` if form data is invalid
    /// - `CoreError::BadRequest` if body exceeds size limit
    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
        self.parse_as_with_limit(body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct LoginForm {
        username: String,
        password: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct SearchForm {
        query: String,
        #[serde(default)]
        page: u32,
        #[serde(default)]
        limit: u32,
    }

    #[tokio::test]
    async fn test_form_parser_new() {
        let parser = FormParser::new();
        assert_eq!(parser.max_size(), DEFAULT_MAX_SIZE);
    }

    #[tokio::test]
    async fn test_form_parser_with_limit() {
        let parser = FormParser::with_limit(512);
        assert_eq!(parser.max_size(), 512);
    }

    #[tokio::test]
    async fn test_form_parser_default() {
        let parser = FormParser::default();
        assert_eq!(parser.max_size(), DEFAULT_MAX_SIZE);
    }

    #[tokio::test]
    async fn test_parse_simple_form() {
        let parser = FormParser::new();
        let body = Bytes::from("username=alice&password=secret123");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("username"), Some(&"alice".to_string()));
        assert_eq!(result.get("password"), Some(&"secret123".to_string()));
    }

    #[tokio::test]
    async fn test_parse_as_simple_form() {
        let body = Bytes::from("username=alice&password=secret123");
        let form: LoginForm = FormParser::parse_as(&body).await.unwrap();

        assert_eq!(form.username, "alice");
        assert_eq!(form.password, "secret123");
    }

    #[tokio::test]
    async fn test_parse_as_with_limit() {
        let parser = FormParser::with_limit(1024);
        let body = Bytes::from("username=bob&password=pass456");
        let form: LoginForm = parser.parse_as_with_limit(&body).await.unwrap();

        assert_eq!(form.username, "bob");
        assert_eq!(form.password, "pass456");
    }

    #[tokio::test]
    async fn test_parse_url_encoded() {
        let parser = FormParser::new();
        let body = Bytes::from("query=hello%20world&page=1&limit=10");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("query"), Some(&"hello world".to_string()));
        assert_eq!(result.get("page"), Some(&"1".to_string()));
        assert_eq!(result.get("limit"), Some(&"10".to_string()));
    }

    #[tokio::test]
    async fn test_parse_special_characters() {
        let parser = FormParser::new();
        let body = Bytes::from("email=user%40example.com&name=John%20Doe");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("email"), Some(&"user@example.com".to_string()));
        assert_eq!(result.get("name"), Some(&"John Doe".to_string()));
    }

    #[tokio::test]
    async fn test_parse_empty_form() {
        let parser = FormParser::new();
        let body = Bytes::from("");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_single_field() {
        let parser = FormParser::new();
        let body = Bytes::from("token=abc123");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("token"), Some(&"abc123".to_string()));
    }

    #[tokio::test]
    async fn test_parse_with_default_values() {
        let body = Bytes::from("query=test");
        let form: SearchForm = FormParser::parse_as(&body).await.unwrap();

        assert_eq!(form.query, "test");
        assert_eq!(form.page, 0); // Default value
        assert_eq!(form.limit, 0); // Default value
    }

    #[tokio::test]
    async fn test_parse_invalid_utf8() {
        let parser = FormParser::new();
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let body = Bytes::from(invalid_bytes);

        let result = parser.parse(&body).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, CoreError::BodyParseError(_)));
    }

    #[tokio::test]
    async fn test_parse_wrong_type() {
        let body = Bytes::from("username=alice&password=secret&extra=field");
        let result: Result<LoginForm, _> = FormParser::parse_as(&body).await;

        // Should succeed - extra fields are ignored by serde
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_missing_required_field() {
        let body = Bytes::from("username=alice");
        let result: Result<LoginForm, _> = FormParser::parse_as(&body).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_size_limit_exceeded() {
        let parser = FormParser::with_limit(10); // Very small limit
        let body = Bytes::from("username=alice&password=secret123"); // Exceeds 10 bytes

        let result = parser.parse(&body).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, CoreError::BadRequest(_)));
    }

    #[tokio::test]
    async fn test_size_limit_exact() {
        let form_data = "a=1&b=2";
        let parser = FormParser::with_limit(form_data.len()); // Exact size
        let body = Bytes::from(form_data);

        let result = parser.parse(&body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_value() {
        let parser = FormParser::new();
        let body = Bytes::from("key1=&key2=value");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("key1"), Some(&"".to_string()));
        assert_eq!(result.get("key2"), Some(&"value".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_equals() {
        let parser = FormParser::new();
        let body = Bytes::from("data=key=value");

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.get("data"), Some(&"key=value".to_string()));
    }

    #[tokio::test]
    async fn test_parser_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FormParser>();
    }

    #[tokio::test]
    async fn test_parser_is_clone() {
        let parser = FormParser::with_limit(1024);
        let cloned = parser.clone();
        assert_eq!(parser.max_size(), cloned.max_size());
    }

    #[tokio::test]
    async fn test_format_form_error() {
        // Test error formatting (internal method)
        let invalid_form = "username=alice"; // Missing password
        let err = serde_urlencoded::from_str::<LoginForm>(invalid_form).unwrap_err();
        let formatted = FormParser::format_form_error(&err);
        assert!(!formatted.is_empty());
    }
}
