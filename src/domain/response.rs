//! Response domain model

use bytes::Bytes;
use std::collections::HashMap;

/// Response body type
#[derive(Clone)]
pub enum ResponseBody {
    /// Static bytes
    Static(Bytes),
    /// Empty body
    Empty,
}

/// Response domain model
///
/// Pure business model with no HTTP protocol knowledge.
pub struct Response {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body
    pub body: ResponseBody,
}

impl Response {
    /// Create a new response
    pub fn new() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: ResponseBody::Empty,
        }
    }

    /// Create a text response
    pub fn text(text: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );

        Self {
            status: 200,
            headers,
            body: ResponseBody::Static(Bytes::from(text.into())),
        }
    }

    /// Create a JSON response
    pub fn json<T: serde::Serialize>(value: &T) -> Self {
        let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Self {
            status: 200,
            headers,
            body: ResponseBody::Static(Bytes::from(body)),
        }
    }

    /// Create an HTML response
    pub fn html(html: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );

        Self {
            status: 200,
            headers,
            body: ResponseBody::Static(Bytes::from(html.into())),
        }
    }

    /// Set status code
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Add a header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
