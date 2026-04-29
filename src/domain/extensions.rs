//! Type-safe extension system

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Type-safe extension bag
#[derive(Clone)]
pub struct Extensions {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create a new extension bag
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Get typed value
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Set typed value
    pub fn set<T: Send + Sync + 'static>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Remove typed value
    ///
    /// Returns the value if it existed, attempting to unwrap it from the Arc.
    /// Returns None if the type doesn't exist or if there are other Arc references.
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|arc| Arc::downcast::<T>(arc).ok())
            .and_then(|arc| Arc::try_unwrap(arc).ok())
    }

    /// Check if type exists
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}
