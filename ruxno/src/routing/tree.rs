//! Router - Radix tree based routing
//!
//! This module provides the `Router` type, which manages route registration
//! and lookup using matchit's radix tree for efficient O(log n) matching.
//!
//! # Design
//!
//! - One radix tree per HTTP method for efficient lookup
//! - Pattern validation at registration time
//! - Duplicate route detection
//! - Parameter extraction during lookup
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::{Router, Method};
//!
//! let mut router = Router::new();
//!
//! // Register routes
//! router.insert(Method::GET, "/users/:id", handler)?;
//! router.insert(Method::POST, "/users", create_handler)?;
//!
//! // Lookup routes
//! if let Some(match_result) = router.lookup(&Method::GET, "/users/123") {
//!     let handler = match_result.handler();
//!     let id = match_result.params().get("id");
//! }
//! ```

use crate::core::{BoxedHandler, CoreError, Method};
use crate::routing::{Match, Params, Pattern, PatternError};
use matchit::Router as MatchitRouter;
use std::collections::HashMap;

/// Router error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum RouterError {
    /// Duplicate route registration
    #[error("Duplicate route: {method} {path}")]
    DuplicateRoute {
        /// HTTP method
        method: Method,
        /// Route path
        path: String,
    },

    /// Invalid pattern syntax
    #[error("Invalid pattern: {0}")]
    InvalidPattern(#[from] PatternError),

    /// Matchit insertion error
    #[error("Failed to insert route: {0}")]
    InsertError(String),
}

impl From<RouterError> for CoreError {
    fn from(err: RouterError) -> Self {
        match err {
            RouterError::DuplicateRoute { method, path } => {
                CoreError::duplicate_route(format!("{:?}", method), path)
            }
            RouterError::InvalidPattern(err) => CoreError::invalid_pattern(err.to_string()),
            RouterError::InsertError(msg) => CoreError::internal(msg),
        }
    }
}

/// Router - Manages route registration and lookup
///
/// Uses matchit's radix tree for efficient route matching. Maintains
/// one tree per HTTP method for optimal performance.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::{Router, Method};
///
/// let mut router = Router::new();
///
/// // Register routes
/// router.insert(Method::GET, "/users", list_users)?;
/// router.insert(Method::GET, "/users/:id", get_user)?;
/// router.insert(Method::POST, "/users", create_user)?;
///
/// // Lookup
/// let match_result = router.lookup(&Method::GET, "/users/123")?;
/// ```
pub struct Router<E = ()> {
    /// Radix trees per HTTP method
    trees: HashMap<Method, MatchitRouter<BoxedHandler<E>>>,

    /// Track registered routes for duplicate detection
    registered: HashMap<(Method, String), ()>,
}

impl<E> Router<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new router
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let router = Router::new();
    /// ```
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            registered: HashMap::new(),
        }
    }

    /// Insert a route into the router
    ///
    /// Validates the pattern, checks for duplicates, and inserts into
    /// the appropriate radix tree.
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method
    /// - `path`: Route pattern (e.g., `/users/:id`)
    /// - `handler`: Handler for this route
    ///
    /// # Errors
    ///
    /// Returns `RouterError` if:
    /// - Pattern is invalid
    /// - Route is already registered
    /// - Matchit insertion fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// router.insert(Method::GET, "/users/:id", handler)?;
    /// router.insert(Method::POST, "/users", create_handler)?;
    /// ```
    pub fn insert(
        &mut self,
        method: Method,
        path: &str,
        handler: BoxedHandler<E>,
    ) -> Result<(), RouterError> {
        // Validate pattern
        let pattern = Pattern::parse(path)?;

        // Check for duplicate
        let key = (method.clone(), path.to_string());
        if self.registered.contains_key(&key) {
            return Err(RouterError::DuplicateRoute {
                method,
                path: path.to_string(),
            });
        }

        // Get or create tree for this method
        let tree = self
            .trees
            .entry(method.clone())
            .or_insert_with(|| MatchitRouter::new());

        // Insert into matchit tree using matchit-compatible pattern
        tree.insert(pattern.matchit_pattern(), handler)
            .map_err(|e| RouterError::InsertError(e.to_string()))?;

        // Mark as registered
        self.registered.insert(key, ());

        Ok(())
    }

    /// Lookup a route in the router
    ///
    /// Searches the radix tree for the given method and path, extracting
    /// any path parameters.
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method
    /// - `path`: Request path
    ///
    /// # Returns
    ///
    /// - `Some(Match)` if route is found
    /// - `None` if no matching route
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some(match_result) = router.lookup(&Method::GET, "/users/123") {
    ///     let handler = match_result.handler();
    ///     let id = match_result.params().get("id");
    /// }
    /// ```
    pub fn lookup(&self, method: &Method, path: &str) -> Option<Match<E>> {
        // Get tree for this method
        let tree = self.trees.get(method)?;

        // Lookup in matchit tree
        let matched = tree.at(path).ok()?;

        // Extract handler (clone the BoxedHandler, not the reference)
        let handler = (*matched.value).clone();

        // Extract parameters
        let params: Params = matched
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Some(Match::new(handler, params))
    }

    /// Check if a route exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if router.has_route(&Method::GET, "/users/:id") {
    ///     println!("Route exists");
    /// }
    /// ```
    pub fn has_route(&self, method: &Method, path: &str) -> bool {
        self.registered
            .contains_key(&(method.clone(), path.to_string()))
    }

    /// Get the number of registered routes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// println!("Total routes: {}", router.len());
    /// ```
    pub fn len(&self) -> usize {
        self.registered.len()
    }

    /// Check if router has no routes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if router.is_empty() {
    ///     println!("No routes registered");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.registered.is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Context, Response};

    // Helper to create a test handler
    fn create_handler<E>() -> BoxedHandler<E>
    where
        E: Send + Sync + 'static,
    {
        let handler = |_ctx: Context<E>| async move { Ok(Response::text("test")) };
        BoxedHandler::new(handler)
    }

    #[test]
    fn test_router_new() {
        let router: Router<()> = Router::new();
        assert!(router.is_empty());
        assert_eq!(router.len(), 0);
    }

    #[test]
    fn test_router_insert_static() {
        let mut router: Router<()> = Router::new();
        let result = router.insert(Method::GET, "/users", create_handler());
        assert!(result.is_ok());
        assert_eq!(router.len(), 1);
    }

    #[test]
    fn test_router_insert_with_param() {
        let mut router: Router<()> = Router::new();
        let result = router.insert(Method::GET, "/users/:id", create_handler());
        assert!(result.is_ok());
        assert_eq!(router.len(), 1);
    }

    #[test]
    fn test_router_insert_multiple_params() {
        let mut router: Router<()> = Router::new();
        let result = router.insert(
            Method::GET,
            "/users/:user_id/posts/:post_id",
            create_handler(),
        );
        assert!(result.is_ok());
        assert_eq!(router.len(), 1);
    }

    #[test]
    fn test_router_insert_wildcard() {
        let mut router: Router<()> = Router::new();
        let result = router.insert(Method::GET, "/api/*", create_handler());
        assert!(result.is_ok());
        assert_eq!(router.len(), 1);
    }

    #[test]
    fn test_router_insert_duplicate_error() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();

        let result = router.insert(Method::GET, "/users", create_handler());
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::DuplicateRoute { method, path } => {
                assert_eq!(method, Method::GET);
                assert_eq!(path, "/users");
            }
            _ => panic!("Expected DuplicateRoute error"),
        }
    }

    #[test]
    fn test_router_insert_same_path_different_methods() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();
        let result = router.insert(Method::POST, "/users", create_handler());
        assert!(result.is_ok());
        assert_eq!(router.len(), 2);
    }

    #[test]
    fn test_router_insert_invalid_pattern() {
        let mut router: Router<()> = Router::new();
        let result = router.insert(Method::GET, "", create_handler());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RouterError::InvalidPattern(_)
        ));
    }

    #[test]
    fn test_router_lookup_static() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/users");
        assert!(match_result.is_some());

        let m = match_result.unwrap();
        assert!(m.params().is_empty());
    }

    #[test]
    fn test_router_lookup_with_param() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users/:id", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/users/123");
        assert!(match_result.is_some());

        let m = match_result.unwrap();
        assert_eq!(m.params().get("id"), Some("123"));
    }

    #[test]
    fn test_router_lookup_multiple_params() {
        let mut router: Router<()> = Router::new();
        router
            .insert(
                Method::GET,
                "/users/:user_id/posts/:post_id",
                create_handler(),
            )
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/users/42/posts/99");
        assert!(match_result.is_some());

        let m = match_result.unwrap();
        assert_eq!(m.params().get("user_id"), Some("42"));
        assert_eq!(m.params().get("post_id"), Some("99"));
    }

    #[test]
    fn test_router_lookup_wildcard() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/api/*", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/api/users/123");
        assert!(match_result.is_some());
    }

    #[test]
    fn test_router_lookup_not_found() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/posts");
        assert!(match_result.is_none());
    }

    #[test]
    fn test_router_lookup_wrong_method() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::POST, "/users");
        assert!(match_result.is_none());
    }

    #[test]
    fn test_router_has_route() {
        let mut router: Router<()> = Router::new();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();

        assert!(router.has_route(&Method::GET, "/users"));
        assert!(!router.has_route(&Method::POST, "/users"));
        assert!(!router.has_route(&Method::GET, "/posts"));
    }

    #[test]
    fn test_router_len() {
        let mut router: Router<()> = Router::new();
        assert_eq!(router.len(), 0);

        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();
        assert_eq!(router.len(), 1);

        router
            .insert(Method::POST, "/users", create_handler())
            .unwrap();
        assert_eq!(router.len(), 2);

        router
            .insert(Method::GET, "/posts", create_handler())
            .unwrap();
        assert_eq!(router.len(), 3);
    }

    #[test]
    fn test_router_is_empty() {
        let mut router: Router<()> = Router::new();
        assert!(router.is_empty());

        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();
        assert!(!router.is_empty());
    }

    #[test]
    fn test_router_multiple_routes() {
        let mut router: Router<()> = Router::new();

        // Register multiple routes
        router.insert(Method::GET, "/", create_handler()).unwrap();
        router
            .insert(Method::GET, "/users", create_handler())
            .unwrap();
        router
            .insert(Method::GET, "/users/:id", create_handler())
            .unwrap();
        router
            .insert(Method::POST, "/users", create_handler())
            .unwrap();
        router
            .insert(Method::GET, "/posts/:id", create_handler())
            .unwrap();

        assert_eq!(router.len(), 5);

        // Lookup each route
        assert!(router.lookup(&Method::GET, "/").is_some());
        assert!(router.lookup(&Method::GET, "/users").is_some());
        assert!(router.lookup(&Method::GET, "/users/123").is_some());
        assert!(router.lookup(&Method::POST, "/users").is_some());
        assert!(router.lookup(&Method::GET, "/posts/456").is_some());
    }

    #[test]
    fn test_router_default() {
        let router: Router<()> = Router::default();
        assert!(router.is_empty());
    }

    #[test]
    fn test_router_with_environment() {
        struct TestEnv {
            value: i32,
        }

        let mut router: Router<TestEnv> = Router::new();
        router
            .insert(Method::GET, "/test", create_handler())
            .unwrap();

        let match_result = router.lookup(&Method::GET, "/test");
        assert!(match_result.is_some());
    }

    #[test]
    fn test_router_error_conversion() {
        let err = RouterError::DuplicateRoute {
            method: Method::GET,
            path: "/users".to_string(),
        };
        let core_err: CoreError = err.into();
        assert!(core_err.is_server_error());
    }
}
