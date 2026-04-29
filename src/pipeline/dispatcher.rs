//! Dispatcher - Unified routing and middleware dispatch
//!
//! This module provides the `Dispatcher`, which orchestrates the entire request
//! handling pipeline: route matching, parameter extraction, context creation,
//! and handler execution with middleware.
//!
//! # Design
//!
//! The Dispatcher is the central orchestrator that:
//! - Stores routes in a `Router` for efficient O(log n) lookup
//! - Pre-computes middleware chains at registration time (not per-request)
//! - Handles 404 (route not found) and 405 (method not allowed) errors
//! - Creates contexts with extracted path parameters
//! - Executes handlers through the middleware chain
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::{Dispatcher, Method};
//!
//! let mut dispatcher = Dispatcher::new(Arc::new(()));
//!
//! // Register routes
//! dispatcher.register_route(Method::GET, "/users/:id", handler)?;
//!
//! // Register global middleware
//! dispatcher.register_middleware(logger_middleware);
//!
//! // Dispatch requests
//! let response = dispatcher.dispatch(request).await?;
//! ```

use crate::core::{BoxedHandler, CoreError, Handler, Method, Middleware};
use crate::domain::{Context, Request, Response};
use crate::pipeline::MiddlewareChain;
use crate::routing::Router;
use std::sync::Arc;

/// Dispatcher - Orchestrates request handling
///
/// The Dispatcher is the central component that ties together routing,
/// middleware, and handler execution. It pre-computes middleware chains
/// at registration time for optimal performance.
///
/// # Examples
///
/// ```rust,ignore
/// let mut dispatcher = Dispatcher::new(Arc::new(env));
///
/// // Register routes
/// dispatcher.register_route(Method::GET, "/", home_handler)?;
/// dispatcher.register_route(Method::GET, "/users/:id", get_user)?;
///
/// // Register global middleware (applied to all routes)
/// dispatcher.register_middleware(logger);
/// dispatcher.register_middleware(cors);
///
/// // Dispatch a request
/// let response = dispatcher.dispatch(request).await?;
/// ```
pub struct Dispatcher<E = ()> {
    /// Router for route matching
    router: Router<E>,

    /// Global middleware (applied to all routes)
    global_middleware: Vec<Arc<dyn Middleware<E>>>,

    /// Environment for dependency injection
    env: Arc<E>,
}

impl<E> Dispatcher<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new dispatcher with the given environment
    ///
    /// # Arguments
    ///
    /// - `env`: Environment for dependency injection (wrapped in Arc)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let dispatcher = Dispatcher::new(Arc::new(()));
    /// ```
    pub fn new(env: Arc<E>) -> Self {
        Self {
            router: Router::new(),
            global_middleware: Vec::new(),
            env,
        }
    }

    /// Register a route with a handler
    ///
    /// The handler is wrapped with all global middleware into a pre-computed
    /// chain at registration time. This means zero per-request overhead for
    /// middleware chain building.
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, etc.)
    /// - `path`: Route pattern (e.g., `/users/:id`)
    /// - `handler`: Handler for this route
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Route registered successfully
    /// - `Err(CoreError)`: Invalid pattern or duplicate route
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// dispatcher.register_route(Method::GET, "/users/:id", handler)?;
    /// dispatcher.register_route(Method::POST, "/users", create_user)?;
    /// ```
    pub fn register_route(
        &mut self,
        method: Method,
        path: &str,
        handler: impl Handler<E>,
    ) -> Result<(), CoreError> {
        // Box the handler
        let boxed_handler = BoxedHandler::new(handler);

        // Build middleware chain (pre-computed at registration time)
        let mut chain = MiddlewareChain::new(boxed_handler);

        // Add global middleware to the chain
        for middleware in &self.global_middleware {
            chain.add(Arc::clone(middleware));
        }

        // Build the pre-computed chain
        let composed_handler = chain.build();

        // Insert into router
        self.router.insert(method, path, composed_handler)?;

        Ok(())
    }

    /// Register global middleware
    ///
    /// Global middleware is applied to all routes. Middleware are applied
    /// in the order they're registered.
    ///
    /// **Important**: Middleware must be registered *before* routes for them
    /// to be included in the route's pre-computed chain.
    ///
    /// # Arguments
    ///
    /// - `middleware`: Middleware to apply to all routes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// dispatcher.register_middleware(logger);
    /// dispatcher.register_middleware(cors);
    /// // Now register routes - they'll include both middleware
    /// dispatcher.register_route(Method::GET, "/", handler)?;
    /// ```
    pub fn register_middleware(&mut self, middleware: impl Middleware<E>) {
        self.global_middleware.push(Arc::new(middleware));
    }

    /// Dispatch a request through the routing and middleware pipeline
    ///
    /// This method:
    /// 1. Looks up the route in the router
    /// 2. Extracts path parameters
    /// 3. Creates a context with the request, params, and environment
    /// 4. Executes the pre-computed handler + middleware chain
    /// 5. Returns the response or error
    ///
    /// # Arguments
    ///
    /// - `req`: The HTTP request to dispatch
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Successful response from handler
    /// - `Err(CoreError)`: 404 (not found), 405 (method not allowed), or handler error
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = dispatcher.dispatch(request).await?;
    /// ```
    pub async fn dispatch(&self, req: Request) -> Result<Response, CoreError> {
        let method = req.method();
        let path = req.path();

        // Lookup route in router
        let matched = self
            .router
            .lookup(method, path)
            .ok_or_else(|| CoreError::not_found(path))?;

        // Extract handler and params
        let (handler, params) = matched.into_parts();

        // Create request with params
        let req_with_params = req.with_params(params);

        // Create context
        let ctx = Context::new(req_with_params, Arc::clone(&self.env));

        // Execute handler (which includes pre-computed middleware chain)
        handler.handle(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ResponseBody;
    use crate::http::Headers;
    use async_trait::async_trait;
    use bytes::Bytes;
    use std::collections::HashMap;

    // Helper to create a minimal test request
    fn create_test_request(method: Method, path: &str) -> Request {
        Request::new(
            method,
            path.parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::new(),
        )
    }

    #[tokio::test]
    async fn test_dispatcher_register_and_dispatch() {
        let mut dispatcher = Dispatcher::new(Arc::new(()));

        // Register a route
        dispatcher
            .register_route(Method::GET, "/hello", |_ctx: Context<()>| async move {
                Ok(Response::text("Hello, World!"))
            })
            .unwrap();

        // Dispatch a request
        let req = create_test_request(Method::GET, "/hello");
        let response = dispatcher.dispatch(req).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Hello, World!"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_with_params() {
        let mut dispatcher = Dispatcher::new(Arc::new(()));

        // Register a route with params
        dispatcher
            .register_route(Method::GET, "/users/:id", |ctx: Context<()>| async move {
                let id = ctx.req.param("id").unwrap();
                Ok(Response::text(format!("User ID: {}", id)))
            })
            .unwrap();

        // Dispatch a request
        let req = create_test_request(Method::GET, "/users/123");
        let response = dispatcher.dispatch(req).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("User ID: 123"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_not_found() {
        let dispatcher: Dispatcher<()> = Dispatcher::new(Arc::new(()));

        // Dispatch to non-existent route
        let req = create_test_request(Method::GET, "/nonexistent");
        let result = dispatcher.dispatch(req).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.status_code(), 404);
            assert!(matches!(error, CoreError::NotFound(_)));
        }
    }

    #[tokio::test]
    async fn test_dispatcher_with_middleware() {
        struct AddHeaderMiddleware {
            name: String,
            value: String,
        }

        #[async_trait]
        impl<E> Middleware<E> for AddHeaderMiddleware
        where
            E: Send + Sync + 'static,
        {
            async fn process(
                &self,
                ctx: Context<E>,
                next: crate::core::Next<E>,
            ) -> Result<Response, CoreError> {
                let mut response = next.run(ctx).await?;
                response.headers_mut().set(&self.name, &self.value).ok();
                Ok(response)
            }
        }

        let mut dispatcher = Dispatcher::new(Arc::new(()));

        // Register middleware BEFORE routes
        dispatcher.register_middleware(AddHeaderMiddleware {
            name: "X-Custom".to_string(),
            value: "middleware-value".to_string(),
        });

        // Register route
        dispatcher
            .register_route(Method::GET, "/test", |_ctx: Context<()>| async move {
                Ok(Response::text("test"))
            })
            .unwrap();

        // Dispatch request
        let req = create_test_request(Method::GET, "/test");
        let response = dispatcher.dispatch(req).await.unwrap();

        // Middleware should have added header
        assert_eq!(
            response.headers().get("X-Custom").unwrap(),
            "middleware-value"
        );
    }

    #[tokio::test]
    async fn test_dispatcher_multiple_routes() {
        let mut dispatcher = Dispatcher::new(Arc::new(()));

        // Register multiple routes
        dispatcher
            .register_route(Method::GET, "/", |_ctx: Context<()>| async move {
                Ok(Response::text("home"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::GET, "/about", |_ctx: Context<()>| async move {
                Ok(Response::text("about"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::POST, "/users", |_ctx: Context<()>| async move {
                Ok(Response::text("create user"))
            })
            .unwrap();

        // Test each route
        let req = create_test_request(Method::GET, "/");
        let response = dispatcher.dispatch(req).await.unwrap();
        match response.body() {
            ResponseBody::Bytes(bytes) => assert_eq!(bytes, &Bytes::from("home")),
            _ => panic!("Expected Bytes body"),
        }

        let req = create_test_request(Method::GET, "/about");
        let response = dispatcher.dispatch(req).await.unwrap();
        match response.body() {
            ResponseBody::Bytes(bytes) => assert_eq!(bytes, &Bytes::from("about")),
            _ => panic!("Expected Bytes body"),
        }

        let req = create_test_request(Method::POST, "/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        match response.body() {
            ResponseBody::Bytes(bytes) => assert_eq!(bytes, &Bytes::from("create user")),
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_with_environment() {
        struct TestEnv {
            value: i32,
        }

        let env = TestEnv { value: 42 };
        let mut dispatcher = Dispatcher::new(Arc::new(env));

        dispatcher
            .register_route(Method::GET, "/env", |ctx: Context<TestEnv>| async move {
                let value = ctx.env().value;
                Ok(Response::text(format!("Value: {}", value)))
            })
            .unwrap();

        let req = create_test_request(Method::GET, "/env");
        let response = dispatcher.dispatch(req).await.unwrap();

        match response.body() {
            ResponseBody::Bytes(bytes) => {
                assert_eq!(bytes, &Bytes::from("Value: 42"));
            }
            _ => panic!("Expected Bytes body"),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_handler_error() {
        let mut dispatcher = Dispatcher::new(Arc::new(()));

        dispatcher
            .register_route(Method::GET, "/error", |_ctx: Context<()>| async move {
                Err(CoreError::bad_request("Invalid input"))
            })
            .unwrap();

        let req = create_test_request(Method::GET, "/error");
        let result = dispatcher.dispatch(req).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.status_code(), 400);
        }
    }

    #[tokio::test]
    async fn test_dispatcher_duplicate_route() {
        let mut dispatcher = Dispatcher::new(Arc::new(()));

        // Register first route
        dispatcher
            .register_route(Method::GET, "/test", |_ctx: Context<()>| async move {
                Ok(Response::text("first"))
            })
            .unwrap();

        // Try to register duplicate
        let result =
            dispatcher.register_route(Method::GET, "/test", |_ctx: Context<()>| async move {
                Ok(Response::text("second"))
            });

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.status_code(), 500);
        }
    }
}
