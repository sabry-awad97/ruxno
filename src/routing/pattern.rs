//! Path pattern parsing and validation
//!
//! This module provides pattern parsing for route matching. Patterns support:
//! - Static segments: `/users`
//! - Dynamic parameters: `/users/:id`
//! - Wildcards: `/api/*`
//! - Catch-all: `*`
//!
//! # Pattern Syntax
//!
//! - **Exact match**: `/users` - matches exactly `/users`
//! - **Parameters**: `/users/:id` - matches `/users/123`, extracts `id=123`
//! - **Multiple params**: `/users/:id/posts/:post_id`
//! - **Wildcards**: `/api/*` - matches `/api/anything/here`
//! - **Catch-all**: `*` - matches any path
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::Pattern;
//!
//! // Parse a pattern
//! let pattern = Pattern::parse("/users/:id")?;
//!
//! // Get matchit-compatible pattern
//! assert_eq!(pattern.matchit_pattern(), "/users/{id}");
//!
//! // Check pattern type
//! assert!(pattern.has_params());
//! ```

use std::collections::HashSet;
use std::fmt;

/// Pattern error types
///
/// Represents errors that can occur during pattern parsing and validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PatternError {
    /// Empty pattern string
    #[error("Pattern cannot be empty")]
    EmptyPattern,

    /// Invalid parameter name (empty or invalid characters)
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Duplicate parameter names in pattern
    #[error("Duplicate parameter name: {0}")]
    DuplicateParameter(String),

    /// Invalid wildcard placement
    #[error("Invalid wildcard: {0}")]
    InvalidWildcard(String),

    /// Invalid pattern syntax
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

/// Pattern type classification
///
/// Used for quick matching decisions and optimization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    /// Exact match: `/users`
    Exact,

    /// Has parameters: `/users/:id`
    Parameterized,

    /// Prefix wildcard: `/api/*`
    PrefixWildcard,

    /// Catch-all: `*`
    CatchAll,
}

/// Path pattern for route matching
///
/// Represents a parsed and validated route pattern. Internally converts
/// `:param` syntax to matchit's `{param}` syntax for efficient matching.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::Pattern;
///
/// // Simple pattern
/// let pattern = Pattern::parse("/users")?;
/// assert_eq!(pattern.original(), "/users");
///
/// // Pattern with parameters
/// let pattern = Pattern::parse("/users/:id/posts/:post_id")?;
/// assert!(pattern.has_params());
/// assert_eq!(pattern.param_names(), &["id", "post_id"]);
/// ```
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Original pattern string (with `:param` syntax)
    original: String,

    /// matchit-compatible pattern (with `{param}` syntax)
    matchit_pattern: String,

    /// Pattern type for quick classification
    pattern_type: PatternType,

    /// Extracted parameter names (in order of appearance)
    param_names: Vec<String>,
}

impl Pattern {
    /// Parse a pattern string into a validated Pattern
    ///
    /// # Pattern Syntax
    ///
    /// - `/users` - Exact match
    /// - `/users/:id` - Parameter (`:id` will be extracted)
    /// - `/users/:id/posts/:post_id` - Multiple parameters
    /// - `/api/*` - Prefix wildcard (matches `/api/anything`)
    /// - `*` - Catch-all (matches any path)
    ///
    /// # Validation
    ///
    /// - Pattern cannot be empty
    /// - Parameter names must be valid identifiers
    /// - No duplicate parameter names
    /// - Wildcards must be at the end
    ///
    /// # Errors
    ///
    /// Returns `PatternError` if the pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Valid patterns
    /// let p1 = Pattern::parse("/users")?;
    /// let p2 = Pattern::parse("/users/:id")?;
    /// let p3 = Pattern::parse("/api/*")?;
    ///
    /// // Invalid patterns
    /// assert!(Pattern::parse("").is_err());  // Empty
    /// assert!(Pattern::parse("/users/:").is_err());  // Empty param name
    /// assert!(Pattern::parse("/users/:id/:id").is_err());  // Duplicate param
    /// ```
    pub fn parse(pattern: &str) -> Result<Self, PatternError> {
        // Validate non-empty
        if pattern.is_empty() {
            return Err(PatternError::EmptyPattern);
        }

        // Determine pattern type
        let pattern_type = Self::classify_pattern(pattern);

        // Extract and validate parameters
        let param_names = Self::extract_params(pattern)?;

        // Validate no duplicate parameters
        Self::validate_no_duplicates(&param_names)?;

        // Validate wildcards
        Self::validate_wildcards(pattern)?;

        // Convert to matchit syntax
        let matchit_pattern = Self::convert_to_matchit(pattern)?;

        Ok(Self {
            original: pattern.to_string(),
            matchit_pattern,
            pattern_type,
            param_names,
        })
    }

    /// Classify the pattern type
    fn classify_pattern(pattern: &str) -> PatternType {
        if pattern == "*" {
            PatternType::CatchAll
        } else if pattern.ends_with("/*") {
            PatternType::PrefixWildcard
        } else if pattern.contains(':') || pattern.contains('*') {
            PatternType::Parameterized
        } else {
            PatternType::Exact
        }
    }

    /// Extract parameter names from pattern
    fn extract_params(pattern: &str) -> Result<Vec<String>, PatternError> {
        let mut params = Vec::new();

        for segment in pattern.split('/') {
            if let Some(param) = segment.strip_prefix(':') {
                // Validate parameter name
                if param.is_empty() {
                    return Err(PatternError::InvalidParameter(
                        "Parameter name cannot be empty".to_string(),
                    ));
                }

                // Validate parameter name contains only valid characters
                if !param
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    return Err(PatternError::InvalidParameter(format!(
                        "Parameter name '{}' contains invalid characters (use only alphanumeric, _, -)",
                        param
                    )));
                }

                params.push(param.to_string());
            }
        }

        Ok(params)
    }

    /// Validate no duplicate parameter names
    fn validate_no_duplicates(params: &[String]) -> Result<(), PatternError> {
        let mut seen = HashSet::new();

        for param in params {
            if !seen.insert(param) {
                return Err(PatternError::DuplicateParameter(param.clone()));
            }
        }

        Ok(())
    }

    /// Validate wildcard placement
    fn validate_wildcards(pattern: &str) -> Result<(), PatternError> {
        // Wildcards must be at the end
        if pattern.contains('*') && pattern != "*" && !pattern.ends_with("/*") {
            // Check if wildcard is in the middle
            if let Some(pos) = pattern.find('*') {
                if pos < pattern.len() - 1 && !pattern[pos..].starts_with("/*") {
                    return Err(PatternError::InvalidWildcard(
                        "Wildcard (*) must be at the end of the pattern".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Convert `:param` syntax to matchit's `{param}` syntax
    fn convert_to_matchit(pattern: &str) -> Result<String, PatternError> {
        // Handle catch-all
        if pattern == "*" {
            return Ok("{*catchall}".to_string());
        }

        // Handle prefix wildcard
        if pattern.ends_with("/*") {
            let prefix = pattern.trim_end_matches("/*");
            if prefix.is_empty() {
                return Ok("{*catchall}".to_string());
            }
            return Ok(format!("{}/{{*catchall}}", prefix));
        }

        // Convert :param to {param}
        let converted = pattern
            .split('/')
            .map(|segment| {
                if let Some(param) = segment.strip_prefix(':') {
                    Ok(format!("{{{}}}", param))
                } else {
                    Ok(segment.to_string())
                }
            })
            .collect::<Result<Vec<_>, PatternError>>()?
            .join("/");

        Ok(converted)
    }

    /// Get the original pattern string
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = Pattern::parse("/users/:id")?;
    /// assert_eq!(pattern.original(), "/users/:id");
    /// ```
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Get the matchit-compatible pattern string
    ///
    /// This is used internally by the router for efficient matching.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = Pattern::parse("/users/:id")?;
    /// assert_eq!(pattern.matchit_pattern(), "/users/{id}");
    /// ```
    pub fn matchit_pattern(&self) -> &str {
        &self.matchit_pattern
    }

    /// Get the pattern type
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = Pattern::parse("/users/:id")?;
    /// assert_eq!(pattern.pattern_type(), &PatternType::Parameterized);
    /// ```
    pub fn pattern_type(&self) -> &PatternType {
        &self.pattern_type
    }

    /// Get parameter names in order of appearance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = Pattern::parse("/users/:id/posts/:post_id")?;
    /// assert_eq!(pattern.param_names(), &["id", "post_id"]);
    /// ```
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }

    /// Check if pattern has parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let p1 = Pattern::parse("/users")?;
    /// assert!(!p1.has_params());
    ///
    /// let p2 = Pattern::parse("/users/:id")?;
    /// assert!(p2.has_params());
    /// ```
    pub fn has_params(&self) -> bool {
        !self.param_names.is_empty()
    }

    /// Check if pattern is a wildcard (prefix or catch-all)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let p1 = Pattern::parse("/api/*")?;
    /// assert!(p1.is_wildcard());
    ///
    /// let p2 = Pattern::parse("*")?;
    /// assert!(p2.is_wildcard());
    /// ```
    pub fn is_wildcard(&self) -> bool {
        matches!(
            self.pattern_type,
            PatternType::PrefixWildcard | PatternType::CatchAll
        )
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Valid Pattern Tests
    // ========================================================================

    #[test]
    fn test_parse_exact() {
        let pattern = Pattern::parse("/users").unwrap();
        assert_eq!(pattern.original(), "/users");
        assert_eq!(pattern.matchit_pattern(), "/users");
        assert_eq!(pattern.pattern_type(), &PatternType::Exact);
        assert!(!pattern.has_params());
        assert!(!pattern.is_wildcard());
    }

    #[test]
    fn test_parse_single_param() {
        let pattern = Pattern::parse("/users/:id").unwrap();
        assert_eq!(pattern.original(), "/users/:id");
        assert_eq!(pattern.matchit_pattern(), "/users/{id}");
        assert_eq!(pattern.pattern_type(), &PatternType::Parameterized);
        assert_eq!(pattern.param_names(), &["id"]);
        assert!(pattern.has_params());
        assert!(!pattern.is_wildcard());
    }

    #[test]
    fn test_parse_multiple_params() {
        let pattern = Pattern::parse("/users/:id/posts/:post_id").unwrap();
        assert_eq!(pattern.original(), "/users/:id/posts/:post_id");
        assert_eq!(pattern.matchit_pattern(), "/users/{id}/posts/{post_id}");
        assert_eq!(pattern.pattern_type(), &PatternType::Parameterized);
        assert_eq!(pattern.param_names(), &["id", "post_id"]);
        assert!(pattern.has_params());
    }

    #[test]
    fn test_parse_param_with_underscore() {
        let pattern = Pattern::parse("/users/:user_id").unwrap();
        assert_eq!(pattern.param_names(), &["user_id"]);
    }

    #[test]
    fn test_parse_param_with_dash() {
        let pattern = Pattern::parse("/users/:user-id").unwrap();
        assert_eq!(pattern.param_names(), &["user-id"]);
    }

    #[test]
    fn test_parse_prefix_wildcard() {
        let pattern = Pattern::parse("/api/*").unwrap();
        assert_eq!(pattern.original(), "/api/*");
        assert_eq!(pattern.matchit_pattern(), "/api/{*catchall}");
        assert_eq!(pattern.pattern_type(), &PatternType::PrefixWildcard);
        assert!(!pattern.has_params());
        assert!(pattern.is_wildcard());
    }

    #[test]
    fn test_parse_catchall() {
        let pattern = Pattern::parse("*").unwrap();
        assert_eq!(pattern.original(), "*");
        assert_eq!(pattern.matchit_pattern(), "{*catchall}");
        assert_eq!(pattern.pattern_type(), &PatternType::CatchAll);
        assert!(!pattern.has_params());
        assert!(pattern.is_wildcard());
    }

    #[test]
    fn test_parse_root() {
        let pattern = Pattern::parse("/").unwrap();
        assert_eq!(pattern.original(), "/");
        assert_eq!(pattern.matchit_pattern(), "/");
        assert_eq!(pattern.pattern_type(), &PatternType::Exact);
    }

    #[test]
    fn test_parse_nested_path() {
        let pattern = Pattern::parse("/api/v1/users").unwrap();
        assert_eq!(pattern.original(), "/api/v1/users");
        assert_eq!(pattern.matchit_pattern(), "/api/v1/users");
        assert_eq!(pattern.pattern_type(), &PatternType::Exact);
    }

    #[test]
    fn test_parse_mixed_params_and_static() {
        let pattern = Pattern::parse("/api/v1/users/:id/profile").unwrap();
        assert_eq!(pattern.param_names(), &["id"]);
        assert_eq!(pattern.matchit_pattern(), "/api/v1/users/{id}/profile");
    }

    // ========================================================================
    // Error Tests
    // ========================================================================

    #[test]
    fn test_parse_empty_error() {
        let result = Pattern::parse("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PatternError::EmptyPattern);
    }

    #[test]
    fn test_parse_empty_param_name() {
        let result = Pattern::parse("/users/:");
        assert!(result.is_err());
        match result.unwrap_err() {
            PatternError::InvalidParameter(msg) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_parse_invalid_param_chars() {
        let result = Pattern::parse("/users/:id@123");
        assert!(result.is_err());
        match result.unwrap_err() {
            PatternError::InvalidParameter(msg) => {
                assert!(msg.contains("invalid characters"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_parse_duplicate_params() {
        let result = Pattern::parse("/users/:id/posts/:id");
        assert!(result.is_err());
        match result.unwrap_err() {
            PatternError::DuplicateParameter(name) => {
                assert_eq!(name, "id");
            }
            _ => panic!("Expected DuplicateParameter error"),
        }
    }

    #[test]
    fn test_parse_wildcard_in_middle() {
        let result = Pattern::parse("/api/*/users");
        assert!(result.is_err());
        match result.unwrap_err() {
            PatternError::InvalidWildcard(msg) => {
                assert!(msg.contains("must be at the end"));
            }
            _ => panic!("Expected InvalidWildcard error"),
        }
    }

    // ========================================================================
    // Display Tests
    // ========================================================================

    #[test]
    fn test_display() {
        let pattern = Pattern::parse("/users/:id").unwrap();
        assert_eq!(format!("{}", pattern), "/users/:id");
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_parse_many_params() {
        let pattern = Pattern::parse("/a/:b/c/:d/e/:f/g/:h").unwrap();
        assert_eq!(pattern.param_names(), &["b", "d", "f", "h"]);
        assert_eq!(pattern.matchit_pattern(), "/a/{b}/c/{d}/e/{f}/g/{h}");
    }

    #[test]
    fn test_parse_numeric_param_name() {
        let pattern = Pattern::parse("/users/:id123").unwrap();
        assert_eq!(pattern.param_names(), &["id123"]);
    }

    #[test]
    fn test_parse_single_char_param() {
        let pattern = Pattern::parse("/users/:i").unwrap();
        assert_eq!(pattern.param_names(), &["i"]);
    }

    #[test]
    fn test_pattern_type_classification() {
        assert_eq!(
            Pattern::parse("/users").unwrap().pattern_type(),
            &PatternType::Exact
        );
        assert_eq!(
            Pattern::parse("/users/:id").unwrap().pattern_type(),
            &PatternType::Parameterized
        );
        assert_eq!(
            Pattern::parse("/api/*").unwrap().pattern_type(),
            &PatternType::PrefixWildcard
        );
        assert_eq!(
            Pattern::parse("*").unwrap().pattern_type(),
            &PatternType::CatchAll
        );
    }

    // ========================================================================
    // Error Display Tests
    // ========================================================================

    #[test]
    fn test_error_display() {
        assert_eq!(
            PatternError::EmptyPattern.to_string(),
            "Pattern cannot be empty"
        );
        assert_eq!(
            PatternError::InvalidParameter("test".to_string()).to_string(),
            "Invalid parameter: test"
        );
        assert_eq!(
            PatternError::DuplicateParameter("id".to_string()).to_string(),
            "Duplicate parameter name: id"
        );
        assert_eq!(
            PatternError::InvalidWildcard("test".to_string()).to_string(),
            "Invalid wildcard: test"
        );
        assert_eq!(
            PatternError::InvalidSyntax("test".to_string()).to_string(),
            "Invalid syntax: test"
        );
    }
}
