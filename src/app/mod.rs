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
use crate::pipeline::Dispatcher;
use std::sync::Arc;

/// Application - Main facade
pub struct App<E = ()> {
    dispatcher: Dispatcher<E>,
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
            dispatcher: Dispatcher::new(Arc::clone(&env)),
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
    pub fn route(&mut self, path: impl Into<String>) -> Route<E> {
        Route::new(self, path)
    }

    /// Register route (internal helper for Route builder)
    pub(crate) fn register_route(&mut self, method: Method, path: &str, handler: impl Handler<E>) {
        // TODO: Register route with dispatcher
        let _ = (method, path, handler);
        todo!("Implement App::register_route")
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

    /// Register middleware with path pattern
    ///
    /// Use `"*"` for global middleware that applies to all routes.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    ///
    /// // Global middleware
    /// app.use_middleware("*", async |ctx: Context, next: Next| {
    ///     println!("Request: {}", ctx.req.path());
    ///     next.run(ctx).await
    /// });
    ///
    /// // Path-specific middleware
    /// app.use_middleware("/api/*", async |ctx: Context, next: Next| {
    ///     // Auth middleware for /api routes
    ///     next.run(ctx).await
    /// });
    /// ```
    pub fn use_middleware(&mut self, pattern: &str, middleware: impl Middleware<E>) -> &mut Self {
        // TODO: Register middleware with pattern
        let _ = (pattern, middleware);
        todo!("Implement App::use_middleware")
    }

    // Server

    /// Start listening
    pub async fn listen(self, addr: &str) -> Result<(), CoreError> {
        // TODO: Create server and listen
        todo!("Implement App::listen")
    }

    /// Dispatch request (internal)
    pub(crate) async fn dispatch(&self, req: Request) -> Result<Response, CoreError> {
        self.dispatcher.dispatch(req).await
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
