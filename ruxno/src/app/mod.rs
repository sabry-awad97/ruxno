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
use std::sync::Arc;

/// Application - Main facade
pub struct App<E = ()> {
    dispatcher: Arc<Dispatcher<E>>,
    env: Arc<E>,
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
            dispatcher: Arc::new(Dispatcher::new(Arc::clone(&env))),
            env,
        }
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
    pub fn route(&mut self, path: impl Into<String>) -> Route<'_, E> {
        Route::new(self, path)
    }

    /// Register route (internal helper for Route builder)
    pub(crate) fn register_route(&mut self, method: Method, path: &str, handler: impl Handler<E>) {
        if let Err(e) = Arc::get_mut(&mut self.dispatcher)
            .expect("Dispatcher should be uniquely owned during route registration")
            .register_route(method.clone(), path, handler)
        {
            panic!(
                "Failed to register route {} {}: {}",
                method.as_str(),
                path,
                e
            );
        }
    }

    /// Register GET route
    pub fn get(&mut self, path: &str, handler: impl Handler<E>) -> &mut Self {
        self.register_route(Method::GET, path, handler);
        self
    }

    /// Register POST route
    pub fn post(&mut self, path: &str, handler: impl Handler<E>) -> &mut Self {
        self.register_route(Method::POST, path, handler);
        self
    }

    /// Register PUT route
    pub fn put(&mut self, path: &str, handler: impl Handler<E>) -> &mut Self {
        self.register_route(Method::PUT, path, handler);
        self
    }

    /// Register DELETE route
    pub fn delete(&mut self, path: &str, handler: impl Handler<E>) -> &mut Self {
        self.register_route(Method::DELETE, path, handler);
        self
    }

    /// Register PATCH route
    pub fn patch(&mut self, path: &str, handler: impl Handler<E>) -> &mut Self {
        self.register_route(Method::PATCH, path, handler);
        self
    }

    // Middleware registration

    /// Register middleware
    ///
    /// Can be used for:
    /// - Global middleware (no pattern): applies to all routes
    /// - Path-specific middleware (with pattern): applies only to matching routes
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
    pub fn r#use(&mut self, middleware: impl Middleware<E>) -> &mut Self {
        Arc::get_mut(&mut self.dispatcher)
            .expect("Dispatcher should be uniquely owned during middleware registration")
            .register_middleware(middleware, None);
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
    pub fn on(
        &mut self,
        method: Method,
        pattern: &str,
        middleware: impl Middleware<E>,
    ) -> &mut Self {
        let opts = MiddlewareOptions::new().for_method(method).on(pattern);
        Arc::get_mut(&mut self.dispatcher)
            .expect("Dispatcher should be uniquely owned during middleware registration")
            .register_middleware(middleware, Some(opts));
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
        Arc::clone(&self.dispatcher).dispatch(req).await
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
        let mut app = App::new();
        // Should not panic
        app.r#use(|_ctx: crate::domain::Context, next: crate::core::Next| async move {
            next.run(_ctx).await
        });
    }

    #[test]
    fn test_app_on() {
        let mut app = App::new();
        // Should not panic
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

        let mut app = App::new();

        // Register global middleware using use()
        app.r#use(TestMiddleware {
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
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert_eq!(response.headers().get("X-Test").unwrap(), "global");

        // Test /public - should also have middleware header (global)
        let req = crate::domain::Request::new(
            Method::GET,
            "/public".parse().unwrap(),
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

        let mut app = App::new();

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
            std::collections::HashMap::new(),
            crate::http::Headers::new(),
            bytes::Bytes::new(),
        );
        let response = app.dispatch(req).await.unwrap();
        assert!(response.headers().get("X-Validated").is_none());
    }
}
