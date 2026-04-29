//! Executor - Handler execution logic
//!
//! This module provides the `Executor` for running handlers. In the current
//! architecture, this is a thin wrapper since `BoxedHandler` already provides
//! the `handle()` method for execution.
//!
//! # Design
//!
//! The Executor serves as a clear abstraction point for handler execution,
//! making it easy to add cross-cutting concerns like:
//! - Execution timeouts
//! - Panic recovery
//! - Execution metrics
//! - Request tracing
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::{Executor, BoxedHandler, Context};
//!
//! let handler = BoxedHandler::new(|ctx: Context| async move {
//!     Ok(ctx.text("Hello!"))
//! });
//!
//! let response = Executor::execute(handler, ctx).await?;
//! ```

use crate::core::{BoxedHandler, CoreError};
use crate::domain::{Context, Response};

/// Executor - Executes handlers
///
/// Provides a clean abstraction for handler execution. Currently a thin
/// wrapper around `BoxedHandler::handle()`, but provides a clear extension
/// point for adding execution-level concerns.
///
/// # Examples
///
/// ```rust,ignore
/// let response = Executor::execute(handler, ctx).await?;
/// ```
pub struct Executor;

impl Executor {
    /// Execute a handler with the given context
    ///
    /// Runs the handler asynchronously and returns the response or error.
    /// The handler is already wrapped in a `BoxedHandler`, so this is
    /// primarily a convenience method that delegates to `BoxedHandler::handle()`.
    ///
    /// # Arguments
    ///
    /// - `handler`: The boxed handler to execute
    /// - `ctx`: The request context
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Successful response from the handler
    /// - `Err(CoreError)`: Error from the handler
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handler = BoxedHandler::new(|ctx: Context| async move {
    ///     Ok(ctx.text("Hello!"))
    /// });
    ///
    /// let response = Executor::execute(handler, ctx).await?;
    /// ```
    pub async fn execute<E>(
        handler: BoxedHandler<E>,
        ctx: Context<E>,
    ) -> Result<Response, CoreError>
    where
        E: Send + Sync + 'static,
    {
        // Execute the handler
        // The BoxedHandler already handles async execution and error conversion
        handler.handle(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Method;
    use crate::domain::{Request, ResponseBody};
    use crate::http::Headers;
    use bytes::Bytes;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Helper to create a minimal test request
    fn create_test_request() -> Request {
        Request::new(
            Method::GET,
            "/test".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::new(),
        )
    }

    #[tokio::test]
    async fn test_executor_execute_success() {
        let handler =
            BoxedHandler::new(|_ctx: Context<()>| async move { Ok(Response::text("success")) });

        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = Executor::execute(handler, ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("success"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_executor_execute_error() {
        let handler = BoxedHandler::new(|_ctx: Context<()>| async move {
            Err(CoreError::bad_request("Invalid input"))
        });

        let ctx = Context::new(create_test_request(), Arc::new(()));
        let result = Executor::execute(handler, ctx).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.status_code(), 400);
        }
    }

    #[tokio::test]
    async fn test_executor_with_context_data() {
        let handler = BoxedHandler::new(|ctx: Context<()>| async move {
            let path = ctx.req.path();
            Ok(Response::text(format!("Path: {}", path)))
        });

        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = Executor::execute(handler, ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Path: /test"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_executor_with_environment() {
        struct TestEnv {
            value: i32,
        }

        let handler = BoxedHandler::new(|ctx: Context<TestEnv>| async move {
            let value = ctx.env().value;
            Ok(Response::text(format!("Value: {}", value)))
        });

        let env = TestEnv { value: 42 };
        let ctx = Context::new(create_test_request(), Arc::new(env));
        let response = Executor::execute(handler, ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Value: 42"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_executor_with_extensions() {
        let handler = BoxedHandler::new(|mut ctx: Context<()>| async move {
            // Set extension
            ctx.set("request-id".to_string());

            // Get extension
            let request_id = ctx.get::<String>().unwrap();
            Ok(Response::text(format!("Request ID: {}", request_id)))
        });

        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = Executor::execute(handler, ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Request ID: request-id"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }
}
