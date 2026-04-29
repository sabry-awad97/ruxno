//! Body parser trait and caching mechanism
//!
//! This module defines the `BodyParser` trait for parsing request bodies
//! into typed data structures. It includes a caching mechanism to ensure
//! that parsing happens only once per request.
//!
//! # Design
//!
//! - **Trait-based**: Extensible parser system via `BodyParser` trait
//! - **Async parsing**: Supports async deserialization
//! - **Type-safe**: Generic over output type `T`
//! - **Cached**: Parse once, reuse result (handled by Request layer)
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::BodyParser;
//! use bytes::Bytes;
//!
//! struct MyParser;
//!
//! #[async_trait::async_trait]
//! impl BodyParser for MyParser {
//!     type Output = MyData;
//!
//!     async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
//!         // Parse body bytes into MyData
//!         todo!()
//!     }
//! }
//! ```

use crate::core::CoreError;
use async_trait::async_trait;
use bytes::Bytes;

/// Body parser trait
///
/// Defines the interface for parsing request bodies into typed data.
/// Implementations should be stateless and reusable.
///
/// # Type Parameters
///
/// - `Output`: The type to parse the body into
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::BodyParser;
///
/// struct JsonParser;
///
/// #[async_trait]
/// impl BodyParser for JsonParser {
///     type Output = serde_json::Value;
///
///     async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
///         serde_json::from_slice(body)
///             .map_err(|e| CoreError::body_parse_error(format!("Invalid JSON: {}", e)))
///     }
/// }
/// ```
#[async_trait]
pub trait BodyParser: Send + Sync {
    /// The output type after parsing
    type Output: Send;

    /// Parse body bytes into the output type
    ///
    /// This method should be stateless and idempotent. The same input
    /// should always produce the same output (or error).
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes to parse
    ///
    /// # Returns
    ///
    /// Returns the parsed output or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Should return `CoreError::BodyParseError` for parsing failures.
    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError>;
}

/// Parse body with automatic content-type detection
///
/// This function provides a convenient way to parse bodies based on
/// the Content-Type header. It delegates to the appropriate parser
/// (JSON, form, text) based on the content type.
///
/// # Arguments
///
/// - `body`: Raw body bytes
/// - `content_type`: Optional Content-Type header value
///
/// # Returns
///
/// Returns the parsed value or an error.
///
/// # Examples
///
/// ```rust,ignore
/// let data: MyStruct = parse_with_content_type(&body, Some("application/json")).await?;
/// ```
pub async fn parse_with_content_type<T>(
    body: &Bytes,
    content_type: Option<&str>,
) -> Result<T, CoreError>
where
    T: serde::de::DeserializeOwned,
{
    match content_type {
        Some(ct) if ct.contains("application/json") => {
            // Use JsonParser for consistent parsing with size limits
            crate::body::JsonParser::parse_as(body).await
        }
        Some(ct) if ct.contains("application/x-www-form-urlencoded") => {
            // Parse as form data
            let text = String::from_utf8(body.to_vec())
                .map_err(|e| CoreError::body_parse_error(format!("Invalid UTF-8: {}", e)))?;
            serde_urlencoded::from_str(&text)
                .map_err(|e| CoreError::body_parse_error(format!("Invalid form data: {}", e)))
        }
        Some(ct) if ct.contains("text/plain") => {
            // Parse as text (if T is String)
            let text = String::from_utf8(body.to_vec())
                .map_err(|e| CoreError::body_parse_error(format!("Invalid UTF-8: {}", e)))?;

            // Try to deserialize from JSON string (for flexibility)
            serde_json::from_str(&format!("\"{}\"", text)).map_err(|e| {
                CoreError::body_parse_error(format!("Cannot parse text as type: {}", e))
            })
        }
        _ => {
            // Default to JSON using JsonParser
            crate::body::JsonParser::parse_as(body).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    // Test parser implementation
    struct TestParser;

    #[async_trait]
    impl BodyParser for TestParser {
        type Output = TestData;

        async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
            serde_json::from_slice(body)
                .map_err(|e| CoreError::body_parse_error(format!("Parse error: {}", e)))
        }
    }

    #[tokio::test]
    async fn test_parser_trait() {
        let parser = TestParser;
        let body = Bytes::from(r#"{"name":"test","value":42}"#);

        let result = parser.parse(&body).await.unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    #[tokio::test]
    async fn test_parser_trait_error() {
        let parser = TestParser;
        let body = Bytes::from("invalid json");

        let result = parser.parse(&body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_with_content_type_json() {
        let body = Bytes::from(r#"{"name":"test","value":42}"#);
        let result: TestData = parse_with_content_type(&body, Some("application/json"))
            .await
            .unwrap();

        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    #[tokio::test]
    async fn test_parse_with_content_type_form() {
        let body = Bytes::from("name=test&value=42");
        let result: TestData =
            parse_with_content_type(&body, Some("application/x-www-form-urlencoded"))
                .await
                .unwrap();

        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    #[tokio::test]
    async fn test_parse_with_content_type_default() {
        let body = Bytes::from(r#"{"name":"test","value":42}"#);
        let result: TestData = parse_with_content_type(&body, None).await.unwrap();

        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    #[tokio::test]
    async fn test_parse_with_content_type_invalid() {
        let body = Bytes::from("invalid");
        let result: Result<TestData, _> =
            parse_with_content_type(&body, Some("application/json")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parser_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestParser>();
    }
}
