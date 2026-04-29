//! Request domain model

use crate::core::{CoreError, Method};
use bytes::Bytes;
use std::collections::HashMap;

/// Request domain model
///
/// Pure business model with no HTTP protocol knowledge.
#[derive(Clone)]
pub struct Request {
    /// HTTP method
    pub method: Method,

    /// Request path
    path: String,

    /// Query parameters
    query: HashMap<String, String>,

    /// Headers
    headers: HashMap<String, String>,

    /// Body bytes
    body: Bytes,

    /// Path parameters (from routing)
    params: HashMap<String, String>,
}

impl Request {
    /// Create a new request
    pub fn new(
        method: Method,
        path: String,
        query: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: Bytes,
    ) -> Self {
        Self {
            method,
            path,
            query,
            headers,
            body,
            params: HashMap::new(),
        }
    }

    /// Get request path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get query parameter
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    /// Get all query parameters
    pub fn query_all(&self) -> &HashMap<String, String> {
        &self.query
    }

    /// Get header
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get path parameter
    pub fn param(&self, key: &str) -> Result<&str, CoreError> {
        self.params
            .get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| CoreError::BadRequest(format!("Missing path parameter: {}", key)))
    }

    /// Get all path parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Set path parameters (internal use by router)
    pub(crate) fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    /// Get body bytes
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    // TODO: Implement body parsing methods
    // pub async fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, CoreError> {
    //     todo!("Implement JSON parsing")
    // }
    //
    // pub async fn text(&self) -> Result<String, CoreError> {
    //     todo!("Implement text parsing")
    // }
    //
    // pub async fn form(&self) -> Result<HashMap<String, String>, CoreError> {
    //     todo!("Implement form parsing")
    // }
}
