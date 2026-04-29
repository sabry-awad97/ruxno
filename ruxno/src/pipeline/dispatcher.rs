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
use crate::routing::{Pattern, Router};
use std::sync::Arc;

/// Middleware registration options
///
/// Configures how middleware should be applied to routes.
///
/// # Examples
///
/// ```rust,ignore
/// // Global middleware (no filters)
/// let opts = MiddlewareOptions::new();
///
/// // Path-specific middleware
/// let opts = MiddlewareOptions::new().on("/api/*");
///
/// // Method + path-specific middleware
/// let opts = MiddlewareOptions::new().for_method(Method::POST).on("/api/*");
/// ```
#[derive(Default)]
pub struct MiddlewareOptions {
    /// Optional method filter (None = all methods)
    method: Option<Method>,

    /// Optional pattern filter (None = all paths)
    pattern: Option<String>,
}

impl MiddlewareOptions {
    /// Create new middleware options with no filters (global)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = MiddlewareOptions::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set method filter
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = MiddlewareOptions::new().for_method(Method::POST);
    /// ```
    pub fn for_method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    /// Set pattern filter
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = MiddlewareOptions::new().on("/api/*");
    /// ```
    pub fn on(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
}

/// Middleware registration entry
///
/// Stores middleware with optional method and pattern filters.
struct MiddlewareEntry<E> {
    /// Optional method filter (None = all methods)
    method: Option<Method>,

    /// Optional pattern filter (None = all paths)
    pattern: Option<Pattern>,

    /// The middleware instance
    middleware: Arc<dyn Middleware<E>>,
}

impl<E> MiddlewareEntry<E> {
    /// Check if this middleware matches the given method and path
    fn matches(&self, method: &Method, path: &str) -> bool {
        // Check method filter
        if let Some(ref filter_method) = self.method {
            if filter_method != method {
                return false;
            }
        }

        // Check pattern filter
        if let Some(ref pattern) = self.pattern {
            // Use matchit for pattern matching
            let mut router = matchit::Router::new();
            if router.insert(pattern.matchit_pattern(), ()).is_ok() {
                return router.at(path).is_ok();
            }
            return false;
        }

        // No filters = matches everything
        true
    }
}

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
///
/// // Register path-specific middleware
/// dispatcher.register_middleware_on("/api/*", auth_middleware);
///
/// // Register method + path-specific middleware
/// dispatcher.register_middleware_for(Method::POST, "/api/*", validation_middleware);
///
/// // Dispatch a request
/// let response = dispatcher.dispatch(request).await?;
/// ```
pub struct Dispatcher<E = ()> {
    /// Router for route matching
    router: Router<E>,

    /// Middleware entries with optional filters
    middleware_entries: Vec<MiddlewareEntry<E>>,

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
            middleware_entries: Vec::new(),
            env,
        }
    }

    /// Register a route with a handler
    ///
    /// The handler is wrapped with all matching middleware into a pre-computed
    /// chain at registration time. This means zero per-request overhead for
    /// middleware chain building.
    ///
    /// Middleware matching rules:
    /// - Global middleware (no filters) always match
    /// - Method-filtered middleware match if method matches
    /// - Pattern-filtered middleware match if pattern matches the route path
    /// - Method + pattern-filtered middleware match if both match
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

        // Add matching middleware to the chain
        for entry in &self.middleware_entries {
            if entry.matches(&method, path) {
                chain.add(Arc::clone(&entry.middleware));
            }
        }

        // Build the pre-computed chain
        let composed_handler = chain.build();

        // Insert into router
        self.router.insert(method, path, composed_handler)?;

        Ok(())
    }

    /// Register middleware with optional filters
    ///
    /// This is the unified middleware registration method that supports:
    /// - Global middleware (no options)
    /// - Path-specific middleware (with pattern)
    /// - Method-specific middleware (with method)
    /// - Method + path-specific middleware (with both)
    ///
    /// Middleware are applied in the order they're registered.
    ///
    /// **Important**: Middleware must be registered *before* routes for them
    /// to be included in the route's pre-computed chain.
    ///
    /// # Arguments
    ///
    /// - `middleware`: Middleware to register
    /// - `options`: Optional filters (method, pattern, or both)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::{Dispatcher, MiddlewareOptions, Method};
    ///
    /// let mut dispatcher = Dispatcher::new(Arc::new(()));
    ///
    /// // Global middleware (no filters)
    /// dispatcher.register_middleware(logger, None);
    ///
    /// // Path-specific middleware
    /// let opts = MiddlewareOptions::new().on("/api/*");
    /// dispatcher.register_middleware(auth, Some(opts));
    ///
    /// // Method-specific middleware
    /// let opts = MiddlewareOptions::new().for_method(Method::POST);
    /// dispatcher.register_middleware(validator, Some(opts));
    ///
    /// // Method + path-specific middleware
    /// let opts = MiddlewareOptions::new()
    ///     .for_method(Method::POST)
    ///     .on("/api/*");
    /// dispatcher.register_middleware(csrf, Some(opts));
    /// ```
    pub fn register_middleware(
        &mut self,
        middleware: impl Middleware<E>,
        options: Option<MiddlewareOptions>,
    ) {
        let options = options.unwrap_or_default();

        // Parse pattern if provided
        let parsed_pattern = options
            .pattern
            .as_ref()
            .and_then(|p| Pattern::parse(p).ok());

        self.middleware_entries.push(MiddlewareEntry {
            method: options.method,
            pattern: parsed_pattern,
            middleware: Arc::new(middleware),
        });
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
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-Custom".to_string(),
                value: "middleware-value".to_string(),
            },
            None,
        );

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

    #[tokio::test]
    async fn test_dispatcher_pattern_middleware() {
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

        // Register pattern-specific middleware BEFORE routes
        let opts = MiddlewareOptions::new().on("/api/*");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-API".to_string(),
                value: "protected".to_string(),
            },
            Some(opts),
        );

        // Register routes
        dispatcher
            .register_route(Method::GET, "/api/users", |_ctx: Context<()>| async move {
                Ok(Response::text("users"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::GET, "/public", |_ctx: Context<()>| async move {
                Ok(Response::text("public"))
            })
            .unwrap();

        // Test /api/users - should have middleware header
        let req = create_test_request(Method::GET, "/api/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-API").unwrap(), "protected");

        // Test /public - should NOT have middleware header
        let req = create_test_request(Method::GET, "/public");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-API").is_none());
    }

    #[tokio::test]
    async fn test_dispatcher_method_pattern_middleware() {
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

        // Register method + pattern-specific middleware BEFORE routes
        let opts = MiddlewareOptions::new()
            .for_method(Method::POST)
            .on("/api/*");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-Validated".to_string(),
                value: "true".to_string(),
            },
            Some(opts),
        );

        // Register routes
        dispatcher
            .register_route(Method::POST, "/api/users", |_ctx: Context<()>| async move {
                Ok(Response::text("create user"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::GET, "/api/users", |_ctx: Context<()>| async move {
                Ok(Response::text("get users"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::POST, "/public", |_ctx: Context<()>| async move {
                Ok(Response::text("public post"))
            })
            .unwrap();

        // Test POST /api/users - should have middleware header
        let req = create_test_request(Method::POST, "/api/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Validated").unwrap(), "true");

        // Test GET /api/users - should NOT have middleware header (wrong method)
        let req = create_test_request(Method::GET, "/api/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-Validated").is_none());

        // Test POST /public - should NOT have middleware header (wrong path)
        let req = create_test_request(Method::POST, "/public");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-Validated").is_none());
    }

    #[tokio::test]
    async fn test_dispatcher_exact_pattern_middleware() {
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

        // Register exact pattern middleware
        let opts = MiddlewareOptions::new().on("/api/users");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-Exact".to_string(),
                value: "match".to_string(),
            },
            Some(opts),
        );

        // Register routes
        dispatcher
            .register_route(Method::GET, "/api/users", |_ctx: Context<()>| async move {
                Ok(Response::text("users"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::GET, "/api/posts", |_ctx: Context<()>| async move {
                Ok(Response::text("posts"))
            })
            .unwrap();

        // Test /api/users - should have middleware header
        let req = create_test_request(Method::GET, "/api/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Exact").unwrap(), "match");

        // Test /api/posts - should NOT have middleware header
        let req = create_test_request(Method::GET, "/api/posts");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-Exact").is_none());
    }

    #[tokio::test]
    async fn test_dispatcher_multiple_middleware_layers() {
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

        // Register multiple middleware layers
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-Global".to_string(),
                value: "all".to_string(),
            },
            None,
        );

        let opts = MiddlewareOptions::new().on("/api/*");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-API".to_string(),
                value: "api".to_string(),
            },
            Some(opts),
        );

        let opts = MiddlewareOptions::new()
            .for_method(Method::POST)
            .on("/api/*");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-POST".to_string(),
                value: "post".to_string(),
            },
            Some(opts),
        );

        // Register route
        dispatcher
            .register_route(Method::POST, "/api/users", |_ctx: Context<()>| async move {
                Ok(Response::text("create"))
            })
            .unwrap();

        // Test - should have all three headers
        let req = create_test_request(Method::POST, "/api/users");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Global").unwrap(), "all");
        assert_eq!(response.headers().get("X-API").unwrap(), "api");
        assert_eq!(response.headers().get("X-POST").unwrap(), "post");
    }

    #[tokio::test]
    async fn test_dispatcher_parameterized_pattern_middleware() {
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

        // Register middleware with parameterized pattern
        let opts = MiddlewareOptions::new().on("/users/:id");
        dispatcher.register_middleware(
            AddHeaderMiddleware {
                name: "X-User".to_string(),
                value: "specific".to_string(),
            },
            Some(opts),
        );

        // Register routes
        dispatcher
            .register_route(Method::GET, "/users/:id", |_ctx: Context<()>| async move {
                Ok(Response::text("user"))
            })
            .unwrap();

        dispatcher
            .register_route(Method::GET, "/posts/:id", |_ctx: Context<()>| async move {
                Ok(Response::text("post"))
            })
            .unwrap();

        // Test /users/123 - should have middleware header
        let req = create_test_request(Method::GET, "/users/123");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-User").unwrap(), "specific");

        // Test /posts/456 - should NOT have middleware header
        let req = create_test_request(Method::GET, "/posts/456");
        let response = dispatcher.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-User").is_none());
    }
}
