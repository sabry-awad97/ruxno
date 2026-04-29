//! Middleware chain builder - Pre-computed onion pattern
//!
//! This module implements middleware chain composition using the "onion pattern",
//! where each middleware wraps the next handler in a nested closure. The key
//! optimization is that chains are **pre-computed at registration time**, not
//! per-request.
//!
//! # Design
//!
//! Traditional middleware chains build the onion on every request:
//! ```text
//! Request → [build chain] → M1 → M2 → M3 → Handler → Response
//! ```
//!
//! Pre-computed chains build once at registration:
//! ```text
//! Registration: M1(M2(M3(Handler))) → BoxedHandler
//! Request → BoxedHandler → Response
//! ```
//!
//! # Performance
//!
//! - **Zero per-request allocation**: Chain is built once, reused forever
//! - **Zero per-request branching**: No dynamic dispatch through middleware list
//! - **Cache-friendly**: Single Arc dereference instead of Vec iteration
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::{MiddlewareChain, BoxedHandler};
//!
//! // Build a chain at registration time
//! let mut chain = MiddlewareChain::new(handler);
//! chain.add(Arc::new(logger_middleware));
//! chain.add(Arc::new(auth_middleware));
//! let composed = chain.build(); // Pre-computed onion
//!
//! // At request time, just call the composed handler
//! let response = composed.handle(ctx).await?;
//! ```

use crate::core::{BoxedHandler, CoreError, Middleware, Next};
use crate::domain::{Context, Response};
use async_trait::async_trait;
use std::sync::Arc;

/// Middleware chain builder
///
/// Builds a pre-computed middleware chain using the onion pattern.
/// Middleware are folded from last to first, wrapping each handler
/// in the next middleware.
///
/// # Examples
///
/// ```rust,ignore
/// let mut chain = MiddlewareChain::new(handler);
/// chain.add(Arc::new(middleware1));
/// chain.add(Arc::new(middleware2));
/// let composed = chain.build(); // M1(M2(handler))
/// ```
pub struct MiddlewareChain<E = ()> {
    /// Middleware stack (applied in order)
    middleware: Vec<Arc<dyn Middleware<E>>>,

    /// Final handler
    handler: BoxedHandler<E>,
}

impl<E> MiddlewareChain<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new chain with a final handler
    ///
    /// # Arguments
    ///
    /// - `handler`: The final handler to execute after all middleware
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let chain = MiddlewareChain::new(handler);
    /// ```
    pub fn new(handler: BoxedHandler<E>) -> Self {
        Self {
            middleware: Vec::new(),
            handler,
        }
    }

    /// Add middleware to the chain
    ///
    /// Middleware are applied in the order they're added:
    /// - First added = outermost layer (runs first)
    /// - Last added = innermost layer (runs last, just before handler)
    ///
    /// # Arguments
    ///
    /// - `middleware`: Middleware to add to the chain
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// chain.add(Arc::new(logger));  // Runs first
    /// chain.add(Arc::new(auth));    // Runs second
    /// chain.add(Arc::new(cors));    // Runs third
    /// // Execution: logger → auth → cors → handler
    /// ```
    pub fn add(&mut self, middleware: Arc<dyn Middleware<E>>) {
        self.middleware.push(middleware);
    }

    /// Build the pre-computed onion
    ///
    /// Folds middleware from last to first, creating nested closures that
    /// form the onion pattern. The result is a single `BoxedHandler` that
    /// can be called directly without any per-request chain building.
    ///
    /// # Returns
    ///
    /// A `BoxedHandler` that represents the entire middleware chain + handler
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut chain = MiddlewareChain::new(handler);
    /// chain.add(Arc::new(m1));
    /// chain.add(Arc::new(m2));
    /// let composed = chain.build(); // m1(m2(handler))
    ///
    /// // At request time:
    /// let response = composed.handle(ctx).await?;
    /// ```
    pub fn build(self) -> BoxedHandler<E> {
        // If no middleware, return handler directly
        if self.middleware.is_empty() {
            return self.handler;
        }

        // Fold middleware from last to first, building the onion
        // Start with the final handler as the innermost layer
        let mut current_handler = self.handler;

        // Iterate in reverse: last middleware wraps handler, second-to-last wraps that, etc.
        for middleware in self.middleware.into_iter().rev() {
            // Capture the current handler and middleware in the closure
            let next_handler = current_handler;
            let mw = middleware;

            // Create a new handler that runs this middleware with the next handler
            let wrapped = move |ctx: Context<E>| {
                let mw = Arc::clone(&mw);
                let next = Next::new(next_handler.clone());
                async move { mw.process(ctx, next).await }
            };

            // Box the wrapped handler for the next iteration
            current_handler = BoxedHandler::new(wrapped);
        }

        current_handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Method, StatusCode};
    use crate::domain::{Request, ResponseBody};
    use bytes::Bytes;
    use http::HeaderMap;
    use std::collections::HashMap;

    // Helper to create a minimal test request (following pattern from domain/request.rs)
    fn create_test_request() -> Request {
        Request::new(
            Method::GET,
            "/test".parse().unwrap(),
            HashMap::new(),
            HeaderMap::new(),
            Bytes::new(),
        )
    }

    // Helper to create a test handler
    fn create_handler<E>() -> BoxedHandler<E>
    where
        E: Send + Sync + 'static,
    {
        BoxedHandler::new(|_ctx: Context<E>| async move { Ok(Response::text("handler")) })
    }

    // Test middleware that adds a header
    struct AddHeaderMiddleware {
        name: String,
        value: String,
    }

    #[async_trait]
    impl<E> Middleware<E> for AddHeaderMiddleware
    where
        E: Send + Sync + 'static,
    {
        async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
            let mut response = next.run(ctx).await?;
            response.headers_mut().insert(
                self.name.parse::<http::header::HeaderName>().unwrap(),
                self.value.parse().unwrap(),
            );
            Ok(response)
        }
    }

    #[tokio::test]
    async fn test_chain_no_middleware() {
        let handler = create_handler();
        let chain = MiddlewareChain::new(handler);
        let composed = chain.build();

        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = composed.handle(ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("handler"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_chain_single_middleware() {
        let handler = create_handler();
        let mut chain = MiddlewareChain::new(handler);

        chain.add(Arc::new(AddHeaderMiddleware {
            name: "X-Test".to_string(),
            value: "value1".to_string(),
        }));

        let composed = chain.build();
        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = composed.handle(ctx).await.unwrap();

        assert_eq!(response.headers().get("X-Test").unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_chain_multiple_middleware() {
        let handler = create_handler();
        let mut chain = MiddlewareChain::new(handler);

        // Add middleware in order
        chain.add(Arc::new(AddHeaderMiddleware {
            name: "X-First".to_string(),
            value: "1".to_string(),
        }));
        chain.add(Arc::new(AddHeaderMiddleware {
            name: "X-Second".to_string(),
            value: "2".to_string(),
        }));
        chain.add(Arc::new(AddHeaderMiddleware {
            name: "X-Third".to_string(),
            value: "3".to_string(),
        }));

        let composed = chain.build();
        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = composed.handle(ctx).await.unwrap();

        // All middleware should have run
        assert_eq!(response.headers().get("X-First").unwrap(), "1");
        assert_eq!(response.headers().get("X-Second").unwrap(), "2");
        assert_eq!(response.headers().get("X-Third").unwrap(), "3");
    }

    #[tokio::test]
    async fn test_chain_middleware_order() {
        // Test that middleware execute in the correct order
        struct OrderMiddleware {
            id: String,
        }

        #[async_trait]
        impl<E> Middleware<E> for OrderMiddleware
        where
            E: Send + Sync + 'static,
        {
            async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
                // Get existing order or start new
                let mut order = ctx.get::<Vec<String>>().cloned().unwrap_or_else(Vec::new);
                order.push(format!("before-{}", self.id));

                // Store updated order
                let mut ctx = ctx;
                ctx.set(order.clone());

                // Call next
                let response = next.run(ctx).await?;

                // After handler - add to response header
                let mut response = response;
                let after_value = format!("after-{}", self.id);
                response.headers_mut().append(
                    "X-Order".parse::<http::header::HeaderName>().unwrap(),
                    after_value.parse().unwrap(),
                );

                Ok(response)
            }
        }

        let handler = BoxedHandler::new(|ctx: Context<()>| async move {
            let order = ctx.get::<Vec<String>>().cloned().unwrap_or_else(Vec::new);

            // Return order in response body
            let body = format!("{:?}", order);
            Ok(Response::text(body))
        });

        let mut chain = MiddlewareChain::new(handler);
        chain.add(Arc::new(OrderMiddleware {
            id: "1".to_string(),
        }));
        chain.add(Arc::new(OrderMiddleware {
            id: "2".to_string(),
        }));
        chain.add(Arc::new(OrderMiddleware {
            id: "3".to_string(),
        }));

        let composed = chain.build();
        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = composed.handle(ctx).await.unwrap();

        // Check body contains before order
        match response.body() {
            ResponseBody::Bytes(bytes) => {
                let body_str = String::from_utf8_lossy(bytes);
                assert!(body_str.contains("before-1"));
                assert!(body_str.contains("before-2"));
                assert!(body_str.contains("before-3"));
            }
            _ => panic!("Expected Bytes body"),
        }

        // Check headers contain after order (in reverse)
        let after_headers: Vec<_> = response
            .headers()
            .get_all("X-Order")
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(after_headers, vec!["after-3", "after-2", "after-1"]);
    }

    #[tokio::test]
    async fn test_chain_short_circuit() {
        // Test that middleware can short-circuit the chain
        struct AuthMiddleware;

        #[async_trait]
        impl<E> Middleware<E> for AuthMiddleware
        where
            E: Send + Sync + 'static,
        {
            async fn process(
                &self,
                ctx: Context<E>,
                _next: Next<E>,
            ) -> Result<Response, CoreError> {
                // Check for auth header
                if ctx.req.header("Authorization").is_none() {
                    return Ok(Response::new()
                        .with_status(StatusCode::UNAUTHORIZED)
                        .with_body("Unauthorized"));
                }
                // Don't call next - short circuit
                Ok(Response::new()
                    .with_status(StatusCode::FORBIDDEN)
                    .with_body("Forbidden"))
            }
        }

        let handler = create_handler();
        let mut chain = MiddlewareChain::new(handler);
        chain.add(Arc::new(AuthMiddleware));

        let composed = chain.build();
        let ctx = Context::new(create_test_request(), Arc::new(()));
        let response = composed.handle(ctx).await.unwrap();

        // Should get 401, not reach handler
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_chain_with_environment() {
        struct TestEnv {
            value: i32,
        }

        let handler = BoxedHandler::new(|ctx: Context<TestEnv>| async move {
            let value = ctx.env().value;
            Ok(Response::text(format!("value: {}", value)))
        });

        let chain = MiddlewareChain::new(handler);
        let composed = chain.build();

        let env = TestEnv { value: 42 };
        let ctx = Context::new(create_test_request(), Arc::new(env));
        let response = composed.handle(ctx).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("value: 42"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }
}
