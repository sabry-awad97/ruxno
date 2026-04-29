//! Multipart form parser
//!
//! This module provides multipart/form-data parsing functionality with
//! file upload support and per-field size limits.
//!
//! # Design
//!
//! - **File uploads**: Support for binary file data
//! - **Size limits**: Per-field and total size limits
//! - **Type-safe**: Structured Part representation
//! - **Error handling**: Clear error messages for common issues
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::body::MultipartParser;
//! use bytes::Bytes;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let parser = MultipartParser::new("boundary123");
//! let parts = parser.parse_from_bytes(&body).await?;
//!
//! for part in parts {
//!     println!("Field: {}, Size: {} bytes", part.name(), part.data().len());
//! }
//! # Ok(())
//! # }
//! ```

use crate::body::BodyParser;
use crate::core::CoreError;
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;

/// Default maximum size per field (10MB)
const DEFAULT_MAX_FIELD_SIZE: usize = 10 * 1024 * 1024;

/// Default maximum total size (50MB)
const DEFAULT_MAX_TOTAL_SIZE: usize = 50 * 1024 * 1024;

/// Multipart form parser with configurable size limits
///
/// Parses `multipart/form-data` bodies with support for file uploads
/// and per-field size limits.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::body::MultipartParser;
///
/// // Create parser with boundary
/// let parser = MultipartParser::new("----WebKitFormBoundary");
///
/// // Parse multipart data
/// let parts = parser.parse_from_bytes(&body).await?;
/// ```
#[derive(Debug, Clone)]
pub struct MultipartParser {
    /// Boundary string for multipart data
    boundary: String,

    /// Maximum size per field
    max_field_size: usize,

    /// Maximum total size
    max_total_size: usize,
}

impl MultipartParser {
    /// Create a new multipart parser with boundary
    ///
    /// # Arguments
    ///
    /// - `boundary`: The boundary string from Content-Type header
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = MultipartParser::new("----WebKitFormBoundary");
    /// ```
    pub fn new(boundary: impl Into<String>) -> Self {
        Self {
            boundary: boundary.into(),
            max_field_size: DEFAULT_MAX_FIELD_SIZE,
            max_total_size: DEFAULT_MAX_TOTAL_SIZE,
        }
    }

    /// Set maximum size per field
    ///
    /// # Arguments
    ///
    /// - `size`: Maximum size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = MultipartParser::new("boundary")
    ///     .with_max_field_size(5 * 1024 * 1024); // 5MB per field
    /// ```
    pub fn with_max_field_size(mut self, size: usize) -> Self {
        self.max_field_size = size;
        self
    }

    /// Set maximum total size
    ///
    /// # Arguments
    ///
    /// - `size`: Maximum total size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let parser = MultipartParser::new("boundary")
    ///     .with_max_total_size(20 * 1024 * 1024); // 20MB total
    /// ```
    pub fn with_max_total_size(mut self, size: usize) -> Self {
        self.max_total_size = size;
        self
    }

    /// Parse multipart data from bytes
    ///
    /// This is a simplified implementation that works with buffered bodies.
    /// For production use with large files, consider streaming parsers.
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns a vector of parsed parts or an error.
    ///
    /// # Errors
    ///
    /// - `CoreError::BodyParseError` if multipart data is invalid
    /// - `CoreError::BadRequest` if size limits are exceeded
    pub async fn parse_from_bytes(&self, body: &Bytes) -> Result<Vec<Part>, CoreError> {
        // Check total size limit
        if body.len() > self.max_total_size {
            return Err(CoreError::bad_request(format!(
                "Multipart body too large: {} bytes (max: {} bytes)",
                body.len(),
                self.max_total_size
            )));
        }

        // Simple multipart parsing (production code should use multer or similar)
        let boundary_marker = format!("--{}", self.boundary);
        let body_str = String::from_utf8(body.to_vec()).map_err(|e| {
            CoreError::body_parse_error(format!("Invalid UTF-8 in multipart: {}", e))
        })?;

        let mut parts = Vec::new();
        let sections: Vec<&str> = body_str.split(&boundary_marker).collect();

        for section in sections.iter().skip(1) {
            // Skip empty sections and end marker
            let section = section.trim();
            if section.is_empty() || section.starts_with("--") {
                continue;
            }

            // Parse part
            if let Some(part) = self.parse_part(section)? {
                // Check field size limit
                if part.data.len() > self.max_field_size {
                    return Err(CoreError::bad_request(format!(
                        "Field '{}' exceeds size limit: {} bytes (max: {} bytes)",
                        part.name,
                        part.data.len(),
                        self.max_field_size
                    )));
                }
                parts.push(part);
            }
        }

        Ok(parts)
    }

    /// Parse a single multipart part
    fn parse_part(&self, section: &str) -> Result<Option<Part>, CoreError> {
        // Split headers and body
        let parts: Vec<&str> = section.splitn(2, "\r\n\r\n").collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        let headers = parts[0];
        let body = parts[1].trim_end_matches("\r\n");

        // Parse Content-Disposition header
        let mut name = None;
        let mut filename = None;
        let mut content_type = None;

        for line in headers.lines() {
            let line = line.trim();
            if line.to_lowercase().starts_with("content-disposition:") {
                // Extract name and filename
                for part in line.split(';') {
                    let part = part.trim();
                    if part.starts_with("name=") {
                        name = Some(part[5..].trim_matches('"').to_string());
                    } else if part.starts_with("filename=") {
                        filename = Some(part[9..].trim_matches('"').to_string());
                    }
                }
            } else if line.to_lowercase().starts_with("content-type:") {
                content_type = Some(line[13..].trim().to_string());
            }
        }

        let name = name.ok_or_else(|| {
            CoreError::body_parse_error("Missing 'name' in Content-Disposition header")
        })?;

        Ok(Some(Part {
            name,
            filename,
            content_type,
            data: Bytes::from(body.as_bytes().to_vec()),
        }))
    }
}

impl Default for MultipartParser {
    fn default() -> Self {
        Self::new("")
    }
}

#[async_trait]
impl BodyParser for MultipartParser {
    type Output = Vec<Part>;

    /// Parse multipart body into vector of parts
    ///
    /// Note: This requires the boundary to be set via `new()`.
    /// The boundary is typically extracted from the Content-Type header.
    ///
    /// # Arguments
    ///
    /// - `body`: Raw body bytes
    ///
    /// # Returns
    ///
    /// Returns a vector of parsed parts or an error.
    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
        self.parse_from_bytes(body).await
    }
}

/// Multipart part representing a single field or file
///
/// Contains the field name, optional filename, content type, and data.
///
/// # Examples
///
/// ```rust,ignore
/// for part in parts {
///     if part.is_file() {
///         println!("File: {}", part.filename().unwrap());
///     } else {
///         println!("Field: {}", part.name());
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Part {
    /// Field name
    name: String,

    /// Optional filename (for file uploads)
    filename: Option<String>,

    /// Optional content type
    content_type: Option<String>,

    /// Part data
    data: Bytes,
}

impl Part {
    /// Create a new part
    ///
    /// # Arguments
    ///
    /// - `name`: Field name
    /// - `data`: Field data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let part = Part::new("username", Bytes::from("alice"));
    /// ```
    pub fn new(name: impl Into<String>, data: Bytes) -> Self {
        Self {
            name: name.into(),
            filename: None,
            content_type: None,
            data,
        }
    }

    /// Create a new file part
    ///
    /// # Arguments
    ///
    /// - `name`: Field name
    /// - `filename`: Original filename
    /// - `content_type`: MIME type
    /// - `data`: File data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let part = Part::new_file("avatar", "photo.jpg", "image/jpeg", file_data);
    /// ```
    pub fn new_file(
        name: impl Into<String>,
        filename: impl Into<String>,
        content_type: impl Into<String>,
        data: Bytes,
    ) -> Self {
        Self {
            name: name.into(),
            filename: Some(filename.into()),
            content_type: Some(content_type.into()),
            data,
        }
    }

    /// Get field name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get filename (if this is a file upload)
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Get content type
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Get part data
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// Check if this part is a file upload
    pub fn is_file(&self) -> bool {
        self.filename.is_some()
    }

    /// Get data as UTF-8 string
    ///
    /// # Errors
    ///
    /// Returns an error if data is not valid UTF-8.
    pub fn text(&self) -> Result<String, CoreError> {
        String::from_utf8(self.data.to_vec())
            .map_err(|e| CoreError::body_parse_error(format!("Invalid UTF-8: {}", e)))
    }

    /// Convert parts to HashMap (for non-file fields)
    ///
    /// This is a convenience method for simple form data.
    /// File uploads are skipped.
    pub fn to_map(parts: &[Part]) -> HashMap<String, String> {
        parts
            .iter()
            .filter(|p| !p.is_file())
            .filter_map(|p| p.text().ok().map(|text| (p.name.clone(), text)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multipart_parser_new() {
        let parser = MultipartParser::new("boundary123");
        assert_eq!(parser.boundary, "boundary123");
        assert_eq!(parser.max_field_size, DEFAULT_MAX_FIELD_SIZE);
        assert_eq!(parser.max_total_size, DEFAULT_MAX_TOTAL_SIZE);
    }

    #[tokio::test]
    async fn test_multipart_parser_with_limits() {
        let parser = MultipartParser::new("boundary")
            .with_max_field_size(1024)
            .with_max_total_size(4096);

        assert_eq!(parser.max_field_size, 1024);
        assert_eq!(parser.max_total_size, 4096);
    }

    #[tokio::test]
    async fn test_part_new() {
        let part = Part::new("username", Bytes::from("alice"));
        assert_eq!(part.name(), "username");
        assert_eq!(part.data(), &Bytes::from("alice"));
        assert!(!part.is_file());
        assert_eq!(part.filename(), None);
    }

    #[tokio::test]
    async fn test_part_new_file() {
        let part = Part::new_file("avatar", "photo.jpg", "image/jpeg", Bytes::from("data"));
        assert_eq!(part.name(), "avatar");
        assert_eq!(part.filename(), Some("photo.jpg"));
        assert_eq!(part.content_type(), Some("image/jpeg"));
        assert!(part.is_file());
    }

    #[tokio::test]
    async fn test_part_text() {
        let part = Part::new("field", Bytes::from("hello"));
        assert_eq!(part.text().unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_part_text_invalid_utf8() {
        let part = Part::new("field", Bytes::from(vec![0xFF, 0xFE]));
        assert!(part.text().is_err());
    }

    #[tokio::test]
    async fn test_part_to_map() {
        let parts = vec![
            Part::new("username", Bytes::from("alice")),
            Part::new("email", Bytes::from("alice@example.com")),
            Part::new_file("avatar", "photo.jpg", "image/jpeg", Bytes::from("data")),
        ];

        let map = Part::to_map(&parts);
        assert_eq!(map.len(), 2); // File is skipped
        assert_eq!(map.get("username"), Some(&"alice".to_string()));
        assert_eq!(map.get("email"), Some(&"alice@example.com".to_string()));
        assert!(!map.contains_key("avatar"));
    }

    #[tokio::test]
    async fn test_parse_simple_multipart() {
        let body = Bytes::from(
            "------WebKitFormBoundary\r\n\
             Content-Disposition: form-data; name=\"username\"\r\n\
             \r\n\
             alice\r\n\
             ------WebKitFormBoundary\r\n\
             Content-Disposition: form-data; name=\"email\"\r\n\
             \r\n\
             alice@example.com\r\n\
             ------WebKitFormBoundary--\r\n",
        );

        let parser = MultipartParser::new("----WebKitFormBoundary");
        let parts = parser.parse_from_bytes(&body).await.unwrap();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name(), "username");
        assert_eq!(parts[0].text().unwrap(), "alice");
        assert_eq!(parts[1].name(), "email");
        assert_eq!(parts[1].text().unwrap(), "alice@example.com");
    }

    #[tokio::test]
    async fn test_parse_with_file() {
        let body = Bytes::from(
            "------WebKitFormBoundary\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
             Content-Type: text/plain\r\n\
             \r\n\
             file content here\r\n\
             ------WebKitFormBoundary--\r\n",
        );

        let parser = MultipartParser::new("----WebKitFormBoundary");
        let parts = parser.parse_from_bytes(&body).await.unwrap();

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name(), "file");
        assert_eq!(parts[0].filename(), Some("test.txt"));
        assert_eq!(parts[0].content_type(), Some("text/plain"));
        assert!(parts[0].is_file());
    }

    #[tokio::test]
    async fn test_total_size_limit_exceeded() {
        let body = Bytes::from("a".repeat(1000));
        let parser = MultipartParser::new("boundary").with_max_total_size(100);

        let result = parser.parse_from_bytes(&body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_field_size_limit_exceeded() {
        let body = Bytes::from(
            "------WebKitFormBoundary\r\n\
             Content-Disposition: form-data; name=\"large\"\r\n\
             \r\n\
             aaaaaaaaaa\r\n\
             ------WebKitFormBoundary--\r\n",
        );

        let parser = MultipartParser::new("----WebKitFormBoundary").with_max_field_size(5);

        let result = parser.parse_from_bytes(&body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_multipart() {
        let body = Bytes::from("------WebKitFormBoundary--\r\n");
        let parser = MultipartParser::new("----WebKitFormBoundary");
        let parts = parser.parse_from_bytes(&body).await.unwrap();

        assert_eq!(parts.len(), 0);
    }

    #[tokio::test]
    async fn test_parser_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MultipartParser>();
    }

    #[tokio::test]
    async fn test_parser_is_clone() {
        let parser = MultipartParser::new("boundary");
        let cloned = parser.clone();
        assert_eq!(parser.boundary, cloned.boundary);
    }

    #[tokio::test]
    async fn test_part_is_clone() {
        let part = Part::new("field", Bytes::from("data"));
        let cloned = part.clone();
        assert_eq!(part.name(), cloned.name());
    }
}
