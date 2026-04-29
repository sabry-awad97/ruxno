//! JSON body parser
//!
//! This module provides JSON parsing functionality with size limits
//! and comprehensive error handling.
//!
//! # Design
//!
//! - **Size limits**: Configurable max body size (default 2MB)
//! - **Type-safe**: Generic parsing via `parse_as<T>()`
//! - **Error handling**: Clear error messages for common issues
//! - **Flexible**: Parse to `serde_json::Value` or specific types
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::JsonParser;
//! use bytes::Bytes;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let body = Bytes::from(r#"{"name":"Alice","age":30}"#);
//!
//! // Parse to specific type
//! let user: User = JsonParser::parse_as(&body).await?;
//!
//! // Parse with size limit
//! let user: User = JsonParser::with_limit(1024).parse_as(&body).await?;
//! # Ok(())
//! # }
//! ```

use crate::body::BodyParser;
use crate::core::CoreError;
use async_trait::async_trait;
use bytes::Bytes;
use serde::de::DeserializeOwned;

/// Default maximum body size (2MB)
const DEFAULT_MAX_SIZE: usize = 2 * 1024 * 1024;

/// JSON parser with configurable size limits
///
/// Parses JSON request bodies with optional size validation.
/// Provides both generic `serde_json::Value` parsing and
/// type-specific parsing via `parse_as<T>()`.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::JsonParser;
///
/// // Default parser (2MB limit)
/// let parser = JsonParser::new();
///
/// // Custom size limit
/// let parser = JsonParser::with_limit(1024 * 1024); // 1MB
///
/// // Parse to specific type
/// let data: MyStruct = JsonParser::parse_as(&body).await?;
/// ```
#[derive(Debug, Clone)]
pub struct JsonParser {
    /// Maximum allowed body size in bytes
    max_size: usize,
}

impl JsonParser {
    /// Create a new JSON parser with default size limit (2MB)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = JsonParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    /// Create a JSON parser with custom size limit
    ///
    /// # Arguments
    ///
    /// - `max_size`: Maximum body size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = JsonParser::with_limit(1024 * 1024); // 1MB limit
    /// ```
    pub fn with_limit(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Get the configured size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = JsonParser::with_limit(1024);
    /// assert_eq!(parser.max_size(), 1024);
    /// ```
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Parse JSON body into specific type (static method)
    ///
    /// Convenience method for parsing without creating a parser instance.
    /// Uses default size limit (2MB).
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
    /// - `CoreError::BodyParseError` if JSON is invalid
    /// - `CoreError::BadRequest` if body exceeds size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let user: User = JsonParser::parse_as(&body).await?;
    /// ```
    pub async fn parse_as<T: DeserializeOwned>(body: &Bytes) -> Result<T, CoreError> {
        Self::new().parse_as_with_limit(body).await
    }

    /// Parse JSON body into specific type with size limit check
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
    /// - `CoreError::BodyParseError` if JSON is invalid
    /// - `CoreError::BadRequest` if body exceeds size limit
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = JsonParser::with_limit(1024);
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

        // Parse JSON
        serde_json::from_slice(body).map_err(|e| {
            CoreError::body_parse_error(format!("Invalid JSON: {}", Self::format_json_error(&e)))
        })
    }

    /// Format JSON error for better user experience
    ///
    /// Extracts useful information from serde_json errors.
    fn format_json_error(error: &serde_json::Error) -> String {
        if error.is_eof() {
            "Unexpected end of JSON input".to_string()
        } else if error.is_syntax() {
            format!("Syntax error at line {}", error.line())
        } else if error.is_data() {
            "Invalid data type or structure".to_string()
        } else {
            error.to_string()
        }
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BodyParser for JsonParser {
    type Output = serde_json::Value;

    /// Parse JSON body into `serde_json::Value`
    ///
    /// This implementation parses the body into a generic JSON value.
    /// For type-specific parsing, use `parse_as<T>()` instead.
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns a `serde_json::Value` or an error.
    ///
    /// # Errors
    ///
    /// - `CoreError::BodyParseError` if JSON is invalid
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
    struct User {
        name: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_json_parser_new() {
        let parser = JsonParser::new();
        assert_eq!(parser.max_size(), DEFAULT_MAX_SIZE);
    }

    #[tokio::test]
    async fn test_json_parser_with_limit() {
        let parser = JsonParser::with_limit(1024);
        assert_eq!(parser.max_size(), 1024);
    }

    #[tokio::test]
    async fn test_json_parser_default() {
        let parser = JsonParser::default();
        assert_eq!(parser.max_size(), DEFAULT_MAX_SIZE);
    }

    #[tokio::test]
    async fn test_parse_valid_json() {
        let parser = JsonParser::new();
        let body = Bytes::from(r#"{"name":"Alice","age":30}"#);

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
    }

    #[tokio::test]
    async fn test_parse_as_valid_json() {
        let body = Bytes::from(r#"{"name":"Alice","age":30}"#);
        let user: User = JsonParser::parse_as(&body).await.unwrap();

        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[tokio::test]
    async fn test_parse_as_with_limit() {
        let parser = JsonParser::with_limit(1024);
        let body = Bytes::from(r#"{"name":"Bob","age":25}"#);
        let user: User = parser.parse_as_with_limit(&body).await.unwrap();

        assert_eq!(user.name, "Bob");
        assert_eq!(user.age, 25);
    }

    #[tokio::test]
    async fn test_parse_invalid_json() {
        let parser = JsonParser::new();
        let body = Bytes::from("not valid json");

        let result = parser.parse(&body).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, CoreError::BodyParseError(_)));
    }

    #[tokio::test]
    async fn test_parse_as_invalid_json() {
        let body = Bytes::from("not valid json");
        let result: Result<User, _> = JsonParser::parse_as(&body).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_incomplete_json() {
        let parser = JsonParser::new();
        let body = Bytes::from(r#"{"name":"Alice""#); // Missing closing brace

        let result = parser.parse(&body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_wrong_type() {
        let body = Bytes::from(r#"{"name":"Alice","age":"not a number"}"#);
        let result: Result<User, _> = JsonParser::parse_as(&body).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_size_limit_exceeded() {
        let parser = JsonParser::with_limit(10); // Very small limit
        let body = Bytes::from(r#"{"name":"Alice","age":30}"#); // Exceeds 10 bytes

        let result = parser.parse(&body).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, CoreError::BadRequest(_)));
    }

    #[tokio::test]
    async fn test_size_limit_exact() {
        let json = r#"{"a":1}"#;
        let parser = JsonParser::with_limit(json.len()); // Exact size
        let body = Bytes::from(json);

        let result = parser.parse(&body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_json_object() {
        let parser = JsonParser::new();
        let body = Bytes::from("{}");

        let result = parser.parse(&body).await.unwrap();
        assert!(result.is_object());
        assert_eq!(result.as_object().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_empty_json_array() {
        let parser = JsonParser::new();
        let body = Bytes::from("[]");

        let result = parser.parse(&body).await.unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_json_array() {
        let parser = JsonParser::new();
        let body = Bytes::from(r#"[1,2,3]"#);

        let result = parser.parse(&body).await.unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_nested_json() {
        let parser = JsonParser::new();
        let body = Bytes::from(r#"{"user":{"name":"Alice","age":30}}"#);

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result["user"]["name"], "Alice");
        assert_eq!(result["user"]["age"], 30);
    }

    #[tokio::test]
    async fn test_format_json_error() {
        // Test error formatting (internal method)
        let invalid_json = "{ invalid }";
        let err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let formatted = JsonParser::format_json_error(&err);
        assert!(!formatted.is_empty());
    }

    #[tokio::test]
    async fn test_parser_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<JsonParser>();
    }

    #[tokio::test]
    async fn test_parser_is_clone() {
        let parser = JsonParser::with_limit(1024);
        let cloned = parser.clone();
        assert_eq!(parser.max_size(), cloned.max_size());
    }
}
