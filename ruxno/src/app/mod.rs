//! Application facade

mod builder;
mod environment;
mod registry;
mod route;

pub use builder::AppBuilder;
pub use environment::Environment;
pub use registry::Registry;
pub use route::Route;

use crate::core::{CoreError, Handler, Method, Middleware};
use crate::domain::{Request, Response};
use crate::pipeline::{Dispatcher, MiddlewareOptions};
use parking_lot::Mutex;
use std::sync::Arc;

/// Application - Main facade
///
/// The App uses a Registry to collect route and middleware registrations,
/// then builds an immutable Dispatcher on first request or when explicitly built.
///
/// Uses interior mutability (Mutex) to allow lazy dispatcher building while
/// being shareable across threads via Arc.
pub struct App<E = ()> {
    inner: Mutex<AppInner<E>>,
    env: Arc<E>,
}

struct AppInner<E> {
    registry: Registry<E>,
    dispatcher: Option<Arc<Dispatcher<E>>>,
}

impl App<()> {
    /// Create new app with no environment
    pub fn new() -> Self {
        Self::with_env(())
    }
}

impl<E> App<E>
where
    E: Send + Sync + 'static,
{
    /// Create new app with environment
    pub fn with_env(env: E) -> Self {
        let env = Arc::new(env);
        Self {
            inner: Mutex::new(AppInner {
                registry: Registry::new(),
                dispatcher: None,
            }),
            env,
        }
    }

    /// Build the dispatcher from the registry
    ///
    /// This consumes the registry and creates an immutable Dispatcher.
    /// Called automatically on first request if not called explicitly.
    fn build_dispatcher(&self) {
        let mut inner = self.inner.lock();

        if inner.dispatcher.is_some() {
            return; // Already built
        }

        let mut dispatcher = Dispatcher::new(Arc::clone(&self.env));

        // Take the registry contents
        let registry = std::mem::take(&mut inner.registry);
        let (routes, middleware) = registry.into_parts();

        // Register all middleware first (order matters for PostRouting)
        for entry in middleware {
            dispatcher.register_middleware_arc(
                entry.phase(),
                entry.middleware(),
                entry.options().cloned(),
            );
        }

        // Register all routes (this pre-computes middleware chains)
        for entry in routes {
            let method = entry.method().clone();
            let path = entry.path().to_string();
            let handler = entry.handler();

            if let Err(e) = dispatcher.register_route_boxed(method.clone(), &path, handler) {
                panic!(
                    "Failed to register route {} {}: {}",
                    method.as_str(),
                    path,
                    e
                );
            }
        }

        inner.dispatcher = Some(Arc::new(dispatcher));
    }

    // Route registration

    /// Create a route builder for chaining multiple methods
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.route("/users")
    ///     .get(async |c: Context| c.text("Get users"))
    ///     .post(async |c: Context| c.text("Create user"));
    /// ```
    pub fn route(&self, path: impl Into<String>) -> Route<'_, E> {
        Route::new(self, path)
    }

    /// Register route (internal helper for Route builder)
    pub(crate) fn register_route(&self, method: Method, path: &str, handler: impl Handler<E>) {
        let mut inner = self.inner.lock();
        inner.registry.register_route(method, path, handler);
    }

    /// Register GET route
    pub fn get(&self, path: &str, handler: impl Handler<E>) -> &Self {
        self.register_route(Method::GET, path, handler);
        self
    }

    /// Register POST route
    pub fn post(&self, path: &str, handler: impl Handler<E>) -> &Self {
        self.register_route(Method::POST, path, handler);
        self
    }

    /// Register PUT route
    pub fn put(&self, path: &str, handler: impl Handler<E>) -> &Self {
        self.register_route(Method::PUT, path, handler);
        self
    }

    /// Register DELETE route
    pub fn delete(&self, path: &str, handler: impl Handler<E>) -> &Self {
        self.register_route(Method::DELETE, path, handler);
        self
    }

    /// Register PATCH route
    pub fn patch(&self, path: &str, handler: impl Handler<E>) -> &Self {
        self.register_route(Method::PATCH, path, handler);
        self
    }

    // Middleware registration

    /// Register pre-routing middleware (runs before routing)
    ///
    /// Pre-routing middleware executes before route matching occurs.
    /// Use this for:
    /// - CORS preflight requests
    /// - Health checks that should bypass routing
    /// - Early request rejection (rate limiting, IP blocking)
    /// - Global request logging
    ///
    /// **Important**: Pre-routing middleware CANNOT access route parameters
    /// because routing hasn't happened yet.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // CORS preflight middleware
    /// app.use_before_routing(|ctx: Context, next: Next| async move {
    ///     if ctx.req.method() == &Method::OPTIONS {
    ///         return Ok(Response::new().with_status_code(204));
    ///     }
    ///     next.run(ctx).await
    /// });
    /// ```
    pub fn use_before_routing(&self, middleware: impl Middleware<E>) -> &Self {
        use crate::pipeline::MiddlewarePhase;
        let mut inner = self.inner.lock();
        inner
            .registry
            .register_middleware(MiddlewarePhase::PreRouting, middleware, None);
        self
    }

    /// Register pre-routing middleware with path filter
    ///
    /// Pre-routing middleware with path filter runs before routing,
    /// but only for requests matching the specified pattern.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // Health check that bypasses routing
    /// app.use_before_routing_on("/health", |ctx: Context, _next: Next| async move {
    ///     Ok(Response::json(&serde_json::json!({"status": "ok"})))
    /// });
    /// ```
    pub fn use_before_routing_on(&self, pattern: &str, middleware: impl Middleware<E>) -> &Self {
        use crate::pipeline::MiddlewarePhase;
        let opts = MiddlewareOptions::new().on(pattern);
        let mut inner = self.inner.lock();
        inner
            .registry
            .register_middleware(MiddlewarePhase::PreRouting, middleware, Some(opts));
        self
    }

    /// Register post-routing middleware (runs after routing)
    ///
    /// Post-routing middleware executes after route matching.
    /// Use this for:
    /// - Authentication and authorization
    /// - Request validation
    /// - Logging with route context
    /// - Response transformation
    ///
    /// **Benefit**: Post-routing middleware HAS access to route parameters
    /// extracted during routing.
    ///
    /// Patterns support:
    /// - Exact match: `/api/users`
    /// - Parameters: `/api/users/:id`
    /// - Wildcards: `/api/*` (matches all paths under `/api`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // Global middleware (no pattern)
    /// app.r#use(|ctx: Context, next: Next| async move {
    ///     println!("Request: {}", ctx.req.path());
    ///     next.run(ctx).await
    /// });
    ///
    /// // Path-specific middleware
    /// app.r#use("/api/*").with(|ctx: Context, next: Next| async move {
    ///     // Auth middleware for /api routes
    ///     next.run(ctx).await
    /// });
    /// ```
    pub fn r#use(&self, middleware: impl Middleware<E>) -> &Self {
        use crate::pipeline::MiddlewarePhase;
        let mut inner = self.inner.lock();
        inner
            .registry
            .register_middleware(MiddlewarePhase::PostRouting, middleware, None);
        self
    }

    /// Register post-routing middleware with path filter
    ///
    /// Post-routing middleware with path filter runs after routing,
    /// but only for requests matching the specified pattern.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // Auth middleware for API routes
    /// app.use_on("/api/*", |ctx: Context, next: Next| async move {
    ///     // Check authentication
    ///     next.run(ctx).await
    /// });
    /// ```
    pub fn use_on(&self, pattern: &str, middleware: impl Middleware<E>) -> &Self {
        use crate::pipeline::MiddlewarePhase;
        let opts = MiddlewareOptions::new().on(pattern);
        let mut inner = self.inner.lock();
        inner
            .registry
            .register_middleware(MiddlewarePhase::PostRouting, middleware, Some(opts));
        self
    }

    /// Register method + path-specific middleware
    ///
    /// Middleware applies only to routes matching both the method and pattern.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // Method + path-specific middleware
    /// app.on(Method::POST, "/api/*", |ctx: Context, next: Next| async move {
    ///     // Validation middleware for POST /api routes
    ///     next.run(ctx).await
    /// });
    ///
    /// // Routes registered after will include the middleware if they match
    /// app.post("/api/users", |ctx: Context| async move {
    ///     Ok(ctx.text("Create user"))  // Includes validation middleware
    /// });
    ///
    /// app.get("/api/users", |ctx: Context| async move {
    ///     Ok(ctx.text("Get users"))  // No validation middleware (GET, not POST)
    /// });
    /// ```
    pub fn on(&self, method: Method, pattern: &str, middleware: impl Middleware<E>) -> &Self {
        use crate::pipeline::MiddlewarePhase;
        let opts = MiddlewareOptions::new().for_method(method).on(pattern);
        let mut inner = self.inner.lock();
        inner
            .registry
            .register_middleware(MiddlewarePhase::PostRouting, middleware, Some(opts));
        self
    }

    // Server

    /// Start listening
    ///
    /// Creates a server and starts listening for HTTP connections.
    /// The server will run until Ctrl+C is pressed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut app = App::new();
    ///     app.get("/", |ctx: Context| async move {
    ///         Ok(ctx.text("Hello, World!"))
    ///     });
    ///
    ///     app.listen("127.0.0.1:3000").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn listen(self, addr: &str) -> Result<(), CoreError> {
        use crate::server::Server;
        let server = Server::new(self);
        server.listen(addr).await
    }

    /// Get a reference to the environment
    pub fn env(&self) -> &Arc<E> {
        &self.env
    }

    /// Dispatch request (internal)
    pub(crate) async fn dispatch(&self, req: Request) -> Result<Response, CoreError> {
        // Build dispatcher if not already built
        self.build_dispatcher();

        // Clone the dispatcher Arc without holding the lock across await
        let dispatcher = {
            let inner = self.inner.lock();
            Arc::clone(inner.dispatcher.as_ref().unwrap())
        };

        // Dispatch through the immutable dispatcher
        dispatcher.dispatch(req).await
    }
}

impl<E> Default for App<E>
where
    E: Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::with_env(E::default())
    }
}

#[cfg(test)]
mod tests {
    use http::Version;

    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(Arc::strong_count(app.env()) >= 1);
    }

    #[test]
    fn test_app_with_env() {
        #[derive(Debug, Clone)]
        struct MyEnv {
            value: i32,
        }

        let env = MyEnv { value: 42 };
        let app = App::with_env(env);
        assert_eq!(app.env().value, 42);
    }

    #[test]
    fn test_app_default() {
        let app = App::<()>::default();
        assert!(Arc::strong_count(app.env()) >= 1);
    }

    #[test]
    fn test_app_env_accessor() {
        let app = App::new();
        let env_ref = app.env();
        assert!(Arc::strong_count(env_ref) >= 1);
    }

    #[test]
    fn test_app_with_custom_env_default() {
        #[derive(Debug, Clone, Default)]
        struct Config {
            port: u16,
        }

        let app = App::<Config>::default();
        assert_eq!(app.env().port, 0);
    }

    #[test]
    fn test_app_use() {
        let app = App::new();
        // Should not panic - using PostRouting phase
        app.r#use(|_ctx: crate::domain::Context, next: crate::core::Next| async move {
            next.run(_ctx).await
        });
    }

    #[test]
    fn test_app_use_before_routing() {
        let app = App::new();
        // Should not panic - using PreRouting phase
        app.use_before_routing(|_ctx: crate::domain::Context, next: crate::core::Next| async move {
            next.run(_ctx).await
        });
    }

    #[test]
    fn test_app_on() {
        let app = App::new();
        // Should not panic - using PostRouting phase
        app.on(
            Method::POST,
            "/api/*",
            |_ctx: crate::domain::Context, next: crate::core::Next| async move {
                next.run(_ctx).await
            },
        );
    }

    #[tokio::test]
    async fn test_app_pattern_middleware_integration() {
        use crate::core::Middleware;
        use crate::domain::Response;
        use async_trait::async_trait;

        struct TestMiddleware {
            header: String,
        }

        #[async_trait]
        impl Middleware<()> for TestMiddleware {
            async fn process(
                &self,
                ctx: crate::domain::Context<()>,
                next: crate::core::Next<()>,
            ) -> Result<Response, CoreError> {
                let mut response = next.run(ctx).await?;
                response.headers_mut().set("X-Test", &self.header).ok();
                Ok(response)
            }
        }

        let app = App::new();

        // Register pre-routing middleware using use_before_routing()
        app.use_before_routing(TestMiddleware {
            header: "global".to_string(),
        });

        // Register routes
        app.get("/api/users", |ctx: crate::domain::Context<()>| async move {
            Ok(ctx.text("users"))
        });

        app.get("/public", |ctx: crate::domain::Context<()>| async move {
            Ok(ctx.text("public"))
        });

        // Test /api/users - should have middleware header
        let req = crate::domain::Request::new(
            Method::GET,
            "/api/users".parse().unwrap(),
            Version::HTTP_11,
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Test").unwrap(), "global");

        // Test /public - should also have middleware header (global pre-routing)
        let req = crate::domain::Request::new(
            Method::GET,
            "/public".parse().unwrap(),
            Version::HTTP_11,
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Test").unwrap(), "global");
    }

    #[tokio::test]
    async fn test_app_method_pattern_middleware_integration() {
        use crate::core::Middleware;
        use crate::domain::Response;
        use async_trait::async_trait;

        struct TestMiddleware {
            header: String,
        }

        #[async_trait]
        impl Middleware<()> for TestMiddleware {
            async fn process(
                &self,
                ctx: crate::domain::Context<()>,
                next: crate::core::Next<()>,
            ) -> Result<Response, CoreError> {
                let mut response = next.run(ctx).await?;
                response.headers_mut().set("X-Validated", &self.header).ok();
                Ok(response)
            }
        }

        let app = App::new();

        // Register method + pattern-specific middleware using on()
        app.on(
            Method::POST,
            "/api/*",
            TestMiddleware {
                header: "true".to_string(),
            },
        );

        // Register routes
        app.post("/api/users", |ctx: crate::domain::Context<()>| async move {
            Ok(ctx.text("create"))
        });

        app.get("/api/users", |ctx: crate::domain::Context<()>| async move {
            Ok(ctx.text("get"))
        });

        // Test POST /api/users - should have middleware header
        let req = crate::domain::Request::new(
            Method::POST,
            "/api/users".parse().unwrap(),
            Version::HTTP_11,
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Validated").unwrap(), "true");

        // Test GET /api/users - should NOT have middleware header (wrong method)
        let req = crate::domain::Request::new(
            Method::GET,
            "/api/users".parse().unwrap(),
            Version::HTTP_11,
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-Validated").is_none());
    }
}
