//! Header utilities

use std::collections::HashMap;

/// Header map wrapper
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    /// Create new headers
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Get header value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    /// Set header value
    pub fn set(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    /// Convert from http::HeaderMap
    pub fn from_header_map(map: &http::HeaderMap) -> Self {
        // TODO: Convert http::HeaderMap to HashMap
        todo!("Implement Headers::from_header_map")
    }

    /// Convert to http::HeaderMap
    pub fn to_header_map(&self) -> http::HeaderMap {
        // TODO: Convert HashMap to http::HeaderMap
        todo!("Implement Headers::to_header_map")
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}
