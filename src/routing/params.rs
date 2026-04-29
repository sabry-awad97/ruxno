//! Path parameters

use std::collections::HashMap;

/// Path parameters extracted from route
#[derive(Debug, Clone, Default)]
pub struct Params {
    inner: HashMap<String, String>,
}

impl Params {
    /// Create new empty params
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Get parameter value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    /// Insert parameter
    pub fn insert(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    /// Get all parameters
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }

    /// Convert to HashMap
    pub fn into_map(self) -> HashMap<String, String> {
        self.inner
    }
}

impl From<HashMap<String, String>> for Params {
    fn from(map: HashMap<String, String>) -> Self {
        Self { inner: map }
    }
}
