//! Pretty JSON middleware
//!
//! Formats JSON responses with configurable indentation for improved readability.
//! Only affects responses with `application/json` content type.
//!
//! # Examples
//!
//! ## Simple Usage
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::pretty_json;
//!
//! let mut app = App::new();
//!
//! // Use default settings (2-space indentation)
//! app.r#use(pretty_json());
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::PrettyJsonMiddleware;
//!
//! let mut app = App::new();
//!
//! // Use 4 spaces for indentation
//! app.r#use(PrettyJsonMiddleware::with_indent(4));
//!
//! // Only enable in debug builds
//! app.r#use(
//!     PrettyJsonMiddleware::new()
//!         .indent(2)
//!         .when(cfg!(debug_assertions))
//! );
//! ```

use async_trait::async_trait;
use bytes::Bytes;
use ruxno::core::{CoreError, Middleware, Next};
use ruxno::domain::{Response, ResponseBody};
use ruxno::Context;

/// Pretty JSON middleware
///
/// Reformats JSON response bodies to be human-readable with proper indentation.
/// Only affects responses with `application/json` content type.
#[derive(Debug, Clone)]
pub struct PrettyJsonMiddleware {
    /// Number of spaces for indentation (default: 2)
    indent: usize,
    /// Whether to enable pretty printing (default: true)
    enabled: bool,
}

impl PrettyJsonMiddleware {
    /// Create a new pretty JSON middleware with default settings (2-space indentation)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::PrettyJsonMiddleware;
    ///
    /// let pretty_json = PrettyJsonMiddleware::new();
    /// ```
    pub fn new() -> Self {
        Self {
            indent: 2,
            enabled: true,
        }
    }

    /// Create with custom indentation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::PrettyJsonMiddleware;
    ///
    /// let pretty_json = PrettyJsonMiddleware::with_indent(4);
    /// ```
    pub fn with_indent(indent: usize) -> Self {
        Self {
            indent,
            enabled: true,
        }
    }

    /// Set the indentation level (builder pattern)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::PrettyJsonMiddleware;
    ///
    /// let pretty_json = PrettyJsonMiddleware::new().indent(4);
    /// ```
    pub fn indent(mut self, spaces: usize) -> Self {
        self.indent = spaces;
        self
    }

    /// Conditionally enable pretty printing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno_middleware::PrettyJsonMiddleware;
    ///
    /// // Only enable in debug builds
    /// let pretty_json = PrettyJsonMiddleware::new()
    ///     .indent(2)
    ///     .when(cfg!(debug_assertions));
    /// ```
    pub fn when(mut self, condition: bool) -> Self {
        self.enabled = condition;
        self
    }

    /// Format JSON with custom indentation
    fn format_json(&self, json_str: &str) -> Result<String, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;

        // Use serde_json's pretty printer (always uses 2 spaces)
        let pretty = serde_json::to_string_pretty(&value)?;

        // Adjust indentation if not using default 2 spaces
        if self.indent != 2 {
            Ok(adjust_indentation(&pretty, self.indent))
        } else {
            Ok(pretty)
        }
    }
}

impl Default for PrettyJsonMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a pretty JSON middleware with default settings (2-space indentation)
///
/// This is a convenience function for creating a `PrettyJsonMiddleware` with default settings.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::App;
/// use ruxno_middleware::pretty_json;
///
/// let mut app = App::new();
///
/// // Simple usage
/// app.r#use(pretty_json());
/// ```
///
/// For custom configuration, use `PrettyJsonMiddleware` directly:
///
/// ```rust,ignore
/// use ruxno::App;
/// use ruxno_middleware::PrettyJsonMiddleware;
///
/// let mut app = App::new();
///
/// app.r#use(PrettyJsonMiddleware::with_indent(4));
/// ```
pub fn pretty_json() -> PrettyJsonMiddleware {
    PrettyJsonMiddleware::new()
}

/// Adjust indentation of pretty-printed JSON
fn adjust_indentation(json: &str, spaces: usize) -> String {
    if spaces == 2 {
        return json.to_string();
    }

    let lines: Vec<&str> = json.lines().collect();
    let mut result = Vec::with_capacity(lines.len());

    for line in lines {
        // Count leading spaces (serde_json uses 2 spaces by default)
        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        let indent_level = leading_spaces / 2;
        let new_indent = " ".repeat(indent_level * spaces);
        let content = line.trim_start();
        result.push(format!("{}{}", new_indent, content));
    }

    result.join("\n")
}

#[async_trait]
impl<E> Middleware<E> for PrettyJsonMiddleware
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        // If disabled, just pass through
        if !self.enabled {
            return next.run(ctx).await;
        }

        // Execute the next handler
        let response = next.run(ctx).await?;

        // Check if response is JSON
        let is_json = response
            .headers()
            .get("content-type")
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false);

        if !is_json {
            return Ok(response);
        }

        // Get the response body as bytes (only for static bodies)
        let body_bytes = match response.body() {
            ResponseBody::Bytes(bytes) => bytes.clone(),
            _ => return Ok(response), // Can't format empty or streaming responses
        };

        let body_str = String::from_utf8_lossy(&body_bytes);

        // Try to format the JSON
        match self.format_json(&body_str) {
            Ok(formatted) => {
                // Create new response with formatted JSON
                Ok(Response::new()
                    .with_status_code(response.status())
                    .with_header("content-type", "application/json")
                    .with_bytes(Bytes::from(formatted)))
            }
            Err(_) => {
                // If JSON parsing fails, return original response
                Ok(response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_json_creation() {
        let middleware = PrettyJsonMiddleware::new();
        assert_eq!(middleware.indent, 2);
        assert!(middleware.enabled);
    }

    #[test]
    fn test_pretty_json_with_indent() {
        let middleware = PrettyJsonMiddleware::with_indent(4);
        assert_eq!(middleware.indent, 4);
        assert!(middleware.enabled);
    }

    #[test]
    fn test_pretty_json_builder() {
        let middleware = PrettyJsonMiddleware::new().indent(8).when(true);
        assert_eq!(middleware.indent, 8);
        assert!(middleware.enabled);
    }

    #[test]
    fn test_pretty_json_conditional() {
        let middleware = PrettyJsonMiddleware::new().when(false);
        assert!(!middleware.enabled);
    }

    #[test]
    fn test_format_json() {
        let middleware = PrettyJsonMiddleware::new();
        let compact = r#"{"name":"John","age":30,"city":"NYC"}"#;
        let formatted = middleware.format_json(compact).unwrap();

        assert!(formatted.contains('\n'));
        assert!(formatted.contains("  ")); // 2-space indent
    }

    #[test]
    fn test_format_json_custom_indent() {
        let middleware = PrettyJsonMiddleware::with_indent(4);
        let compact = r#"{"name":"John","nested":{"key":"value"}}"#;
        let formatted = middleware.format_json(compact).unwrap();

        assert!(formatted.contains('\n'));
        assert!(formatted.contains("    ")); // 4-space indent
    }

    #[test]
    fn test_adjust_indentation() {
        let json_2_spaces = "{\n  \"key\": \"value\"\n}";
        let json_4_spaces = adjust_indentation(json_2_spaces, 4);

        assert_eq!(json_4_spaces, "{\n    \"key\": \"value\"\n}");
    }

    #[test]
    fn test_adjust_indentation_no_change() {
        let json = "{\n  \"key\": \"value\"\n}";
        let result = adjust_indentation(json, 2);

        assert_eq!(result, json);
    }

    #[test]
    fn test_pretty_json_function() {
        let middleware = pretty_json();
        assert_eq!(middleware.indent, 2);
        assert!(middleware.enabled);
    }
}
