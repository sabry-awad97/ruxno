//! Router - Radix tree based routing

use crate::core::{BoxedHandler, Method};
use crate::routing::{Match, Pattern, PatternError};
use matchit::Router as MatchitRouter;
use std::collections::HashMap;

/// Router error
#[derive(Debug)]
pub enum RouterError {
    /// Duplicate route
    DuplicateRoute { method: Method, path: String },
    /// Invalid pattern
    InvalidPattern(PatternError),
}

/// Router - Manages route registration and lookup
pub struct Router<E = ()> {
    /// Radix trees per HTTP method
    trees: HashMap<Method, MatchitRouter<BoxedHandler<E>>>,
}

impl<E> Router<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new router
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
        }
    }

    /// Insert a route
    pub fn insert(
        &mut self,
        method: Method,
        path: &str,
        handler: BoxedHandler<E>,
    ) -> Result<(), RouterError> {
        // TODO: Validate pattern
        // TODO: Check for duplicates
        // TODO: Insert into radix tree
        todo!("Implement Router::insert")
    }

    /// Lookup a route
    pub fn lookup(&self, method: &Method, path: &str) -> Option<Match<E>> {
        // TODO: Lookup in radix tree
        // TODO: Extract parameters
        // TODO: Return Match
        todo!("Implement Router::lookup")
    }
}

impl<E> Default for Router<E>
where
    E: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
