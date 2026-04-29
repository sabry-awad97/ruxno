//! Path parameters
//!
//! This module provides the `Params` type for storing path parameters
//! extracted from route matching. Parameters are stored as key-value pairs
//! where both keys and values are strings.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::Params;
//!
//! // Create from vector
//! let params = Params::from(vec![
//!     ("id".to_string(), "123".to_string()),
//!     ("name".to_string(), "john".to_string()),
//! ]);
//!
//! // Access parameters
//! assert_eq!(params.get("id"), Some("123"));
//! assert_eq!(params.len(), 2);
//!
//! // Iterate over parameters
//! for (key, value) in params.iter() {
//!     println!("{} = {}", key, value);
//! }
//! ```

use std::collections::HashMap;

/// Path parameters extracted from route matching
///
/// Stores key-value pairs of path parameters extracted during route matching.
/// For example, a route pattern `/users/:id` matched against `/users/123`
/// would extract `id=123`.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::Params;
///
/// let mut params = Params::new();
/// params.insert("id".to_string(), "123".to_string());
///
/// assert_eq!(params.get("id"), Some("123"));
/// assert_eq!(params.len(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    inner: HashMap<String, String>,
}

impl Params {
    /// Create new empty params
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::new();
    /// assert!(params.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Get parameter value by key
    ///
    /// Returns `None` if the parameter doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![("id".to_string(), "123".to_string())]);
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("name"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    /// Insert a parameter
    ///
    /// If the key already exists, the value is updated.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut params = Params::new();
    /// params.insert("id".to_string(), "123".to_string());
    /// assert_eq!(params.get("id"), Some("123"));
    ///
    /// // Update existing key
    /// params.insert("id".to_string(), "456".to_string());
    /// assert_eq!(params.get("id"), Some("456"));
    /// ```
    pub fn insert(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    /// Check if a parameter exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![("id".to_string(), "123".to_string())]);
    /// assert!(params.contains_key("id"));
    /// assert!(!params.contains_key("name"));
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Get the number of parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![
    ///     ("id".to_string(), "123".to_string()),
    ///     ("name".to_string(), "john".to_string()),
    /// ]);
    /// assert_eq!(params.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if there are no parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::new();
    /// assert!(params.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over parameters
    ///
    /// Returns an iterator over `(&String, &String)` pairs.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![
    ///     ("id".to_string(), "123".to_string()),
    ///     ("name".to_string(), "john".to_string()),
    /// ]);
    ///
    /// for (key, value) in params.iter() {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }

    /// Get all parameter keys
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![
    ///     ("id".to_string(), "123".to_string()),
    ///     ("name".to_string(), "john".to_string()),
    /// ]);
    ///
    /// let keys: Vec<_> = params.keys().collect();
    /// assert!(keys.contains(&&"id".to_string()));
    /// assert!(keys.contains(&&"name".to_string()));
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Get all parameter values
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![
    ///     ("id".to_string(), "123".to_string()),
    ///     ("name".to_string(), "john".to_string()),
    /// ]);
    ///
    /// let values: Vec<_> = params.values().collect();
    /// assert!(values.contains(&&"123".to_string()));
    /// assert!(values.contains(&&"john".to_string()));
    /// ```
    pub fn values(&self) -> impl Iterator<Item = &String> {
        self.inner.values()
    }

    /// Convert to HashMap
    ///
    /// Consumes the `Params` and returns the inner `HashMap`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![("id".to_string(), "123".to_string())]);
    /// let map = params.into_map();
    /// assert_eq!(map.get("id"), Some(&"123".to_string()));
    /// ```
    pub fn into_map(self) -> HashMap<String, String> {
        self.inner
    }
}

// Conversion traits

impl From<HashMap<String, String>> for Params {
    fn from(map: HashMap<String, String>) -> Self {
        Self { inner: map }
    }
}

impl From<Vec<(String, String)>> for Params {
    fn from(vec: Vec<(String, String)>) -> Self {
        Self {
            inner: vec.into_iter().collect(),
        }
    }
}

impl FromIterator<(String, String)> for Params {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Params {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Params {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_new() {
        let params = Params::new();
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_params_insert_and_get() {
        let mut params = Params::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("name".to_string(), "john".to_string());

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("john"));
        assert_eq!(params.get("age"), None);
    }

    #[test]
    fn test_params_insert_overwrites() {
        let mut params = Params::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("id".to_string(), "456".to_string());

        assert_eq!(params.get("id"), Some("456"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_params_contains_key() {
        let mut params = Params::new();
        params.insert("id".to_string(), "123".to_string());

        assert!(params.contains_key("id"));
        assert!(!params.contains_key("name"));
    }

    #[test]
    fn test_params_len() {
        let mut params = Params::new();
        assert_eq!(params.len(), 0);

        params.insert("id".to_string(), "123".to_string());
        assert_eq!(params.len(), 1);

        params.insert("name".to_string(), "john".to_string());
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_params_is_empty() {
        let mut params = Params::new();
        assert!(params.is_empty());

        params.insert("id".to_string(), "123".to_string());
        assert!(!params.is_empty());
    }

    #[test]
    fn test_params_iter() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let mut count = 0;
        for (key, value) in params.iter() {
            count += 1;
            assert!(key == "id" || key == "name");
            assert!(value == "123" || value == "john");
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_params_keys() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let keys: Vec<_> = params.keys().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&"id".to_string()));
        assert!(keys.contains(&&"name".to_string()));
    }

    #[test]
    fn test_params_values() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let values: Vec<_> = params.values().collect();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&&"123".to_string()));
        assert!(values.contains(&&"john".to_string()));
    }

    #[test]
    fn test_params_into_map() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let map = params.into_map();
        assert_eq!(map.get("id"), Some(&"123".to_string()));
        assert_eq!(map.get("name"), Some(&"john".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_params_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "john".to_string());

        let params = Params::from(map);
        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("john"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_params_from_vec() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("john"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_params_from_iterator() {
        let vec = vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ];

        let params: Params = vec.into_iter().collect();
        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("john"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_params_into_iterator() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let mut count = 0;
        for (key, value) in params {
            count += 1;
            assert!(key == "id" || key == "name");
            assert!(value == "123" || value == "john");
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_params_ref_into_iterator() {
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let mut count = 0;
        for (key, value) in &params {
            count += 1;
            assert!(key == "id" || key == "name");
            assert!(value == "123" || value == "john");
        }
        assert_eq!(count, 2);

        // params is still usable
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_params_clone() {
        let params1 = Params::from(vec![("id".to_string(), "123".to_string())]);
        let params2 = params1.clone();

        assert_eq!(params1.get("id"), Some("123"));
        assert_eq!(params2.get("id"), Some("123"));
    }

    #[test]
    fn test_params_default() {
        let params = Params::default();
        assert!(params.is_empty());
    }

    #[test]
    fn test_params_equality() {
        let params1 = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let params2 = Params::from(vec![
            ("name".to_string(), "john".to_string()),
            ("id".to_string(), "123".to_string()),
        ]);

        assert_eq!(params1, params2);
    }

    #[test]
    fn test_params_empty_value() {
        let mut params = Params::new();
        params.insert("empty".to_string(), "".to_string());

        assert_eq!(params.get("empty"), Some(""));
        assert_eq!(params.len(), 1);
    }
}
