//! Route matching result
//!
//! This module defines the `Match` type, which represents the result of
//! successful route matching. It contains the matched handler and any
//! extracted path parameters.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::{Match, Params, BoxedHandler};
//!
//! // Create a match result
//! let params = Params::from(vec![("id".to_string(), "123".to_string())]);
//! let match_result = Match::new(handler, params);
//!
//! // Access handler and params
//! let handler = match_result.handler();
//! let params = match_result.params();
//! ```

use crate::core::BoxedHandler;
use crate::routing::Params;

/// Result of successful route matching
///
/// Contains the matched handler and any path parameters extracted
/// during the matching process.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::{Match, Params};
///
/// // After route matching
/// let match_result = router.lookup(method, path)?;
///
/// // Access the handler
/// let handler = match_result.handler();
///
/// // Access extracted parameters
/// let params = match_result.params();
/// let id = params.get("id");
/// ```
pub struct Match<E = ()> {
    /// Matched handler
    handler: BoxedHandler<E>,

    /// Extracted path parameters
    params: Params,
}

impl<E> Match<E> {
    /// Create a new match result
    ///
    /// # Arguments
    ///
    /// - `handler`: The matched handler
    /// - `params`: Extracted path parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = Params::from(vec![("id".to_string(), "123".to_string())]);
    /// let match_result = Match::new(handler, params);
    /// ```
    pub fn new(handler: BoxedHandler<E>, params: Params) -> Self {
        Self { handler, params }
    }

    /// Get reference to the matched handler
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handler = match_result.handler();
    /// let response = handler.handle(ctx).await?;
    /// ```
    pub fn handler(&self) -> &BoxedHandler<E> {
        &self.handler
    }

    /// Get reference to the extracted parameters
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = match_result.params();
    /// let id = params.get("id");
    /// ```
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// Get mutable reference to the parameters
    ///
    /// Useful for modifying parameters before passing to handler.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = match_result.params_mut();
    /// params.insert("extra".to_string(), "value".to_string());
    /// ```
    pub fn params_mut(&mut self) -> &mut Params {
        &mut self.params
    }

    /// Consume the match and return handler and params
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let (handler, params) = match_result.into_parts();
    /// ```
    pub fn into_parts(self) -> (BoxedHandler<E>, Params) {
        (self.handler, self.params)
    }

    /// Take the handler, leaving params behind
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handler = match_result.into_handler();
    /// ```
    pub fn into_handler(self) -> BoxedHandler<E> {
        self.handler
    }

    /// Take the params, leaving handler behind
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = match_result.into_params();
    /// ```
    pub fn into_params(self) -> Params {
        self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Context, Response};

    // Helper function to create a test handler
    fn create_test_handler<E>() -> BoxedHandler<E>
    where
        E: Send + Sync + 'static,
    {
        let handler = |_ctx: Context<E>| async move { Ok(Response::text("test")) };
        BoxedHandler::new(handler)
    }

    #[test]
    fn test_match_new() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let match_result = Match::new(handler, params);

        assert_eq!(match_result.params().get("id"), Some("123"));
    }

    #[test]
    fn test_match_handler() {
        let handler = create_test_handler::<()>();
        let params = Params::new();

        let match_result = Match::new(handler, params);
        let _handler_ref = match_result.handler();

        // Handler reference is accessible
        assert!(true);
    }

    #[test]
    fn test_match_params() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![
            ("id".to_string(), "123".to_string()),
            ("name".to_string(), "john".to_string()),
        ]);

        let match_result = Match::new(handler, params);
        let params_ref = match_result.params();

        assert_eq!(params_ref.get("id"), Some("123"));
        assert_eq!(params_ref.get("name"), Some("john"));
        assert_eq!(params_ref.len(), 2);
    }

    #[test]
    fn test_match_params_mut() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let mut match_result = Match::new(handler, params);

        // Modify params
        match_result
            .params_mut()
            .insert("extra".to_string(), "value".to_string());

        assert_eq!(match_result.params().get("id"), Some("123"));
        assert_eq!(match_result.params().get("extra"), Some("value"));
        assert_eq!(match_result.params().len(), 2);
    }

    #[test]
    fn test_match_into_parts() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let match_result = Match::new(handler, params);
        let (_handler, params) = match_result.into_parts();

        assert_eq!(params.get("id"), Some("123"));
    }

    #[test]
    fn test_match_into_handler() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let match_result = Match::new(handler, params);
        let _handler = match_result.into_handler();

        // Handler is extracted
        assert!(true);
    }

    #[test]
    fn test_match_into_params() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let match_result = Match::new(handler, params);
        let params = match_result.into_params();

        assert_eq!(params.get("id"), Some("123"));
    }

    #[test]
    fn test_match_empty_params() {
        let handler = create_test_handler::<()>();
        let params = Params::new();

        let match_result = Match::new(handler, params);

        assert!(match_result.params().is_empty());
        assert_eq!(match_result.params().len(), 0);
    }

    #[test]
    fn test_match_with_environment() {
        struct TestEnv {
            value: i32,
        }

        let handler = create_test_handler::<TestEnv>();
        let params = Params::from(vec![("id".to_string(), "123".to_string())]);

        let match_result = Match::new(handler, params);

        assert_eq!(match_result.params().get("id"), Some("123"));
    }

    #[test]
    fn test_match_multiple_params() {
        let handler = create_test_handler::<()>();
        let params = Params::from(vec![
            ("user_id".to_string(), "42".to_string()),
            ("post_id".to_string(), "99".to_string()),
            ("comment_id".to_string(), "7".to_string()),
        ]);

        let match_result = Match::new(handler, params);
        let params_ref = match_result.params();

        assert_eq!(params_ref.get("user_id"), Some("42"));
        assert_eq!(params_ref.get("post_id"), Some("99"));
        assert_eq!(params_ref.get("comment_id"), Some("7"));
        assert_eq!(params_ref.len(), 3);
    }
}
