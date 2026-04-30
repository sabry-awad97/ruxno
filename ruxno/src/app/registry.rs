//! Route and middleware registry
//!
//! The Registry collects route and middleware registrations before building
//! the Dispatcher. This provides a clean separation between registration
//! (mutable, happens during app setup) and dispatch (immutable, happens
//! during request handling).

use crate::core::{BoxedHandler, Handler, Method, Middleware};
use crate::pipeline::{MiddlewareOptions, MiddlewarePhase};
use std::sync::Arc;

/// Registry for routes and middleware
///
/// Collects route and middleware registrations during app setup,
/// then builds a Dispatcher for request handling.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::app::Registry;
/// use ruxno::core::Method;
///
/// let mut registry = Registry::new();
///
/// // Register routes
/// registry.register_route(Method::GET, "/users", handler);
///
/// // Register middleware
/// registry.register_middleware(MiddlewarePhase::PostRouting, middleware, None);
/// ```
pub struct Registry<E = ()> {
    routes: Vec<RouteEntry<E>>,
    middleware: Vec<MiddlewareEntry<E>>,
}

/// Route registration entry
pub(crate) struct RouteEntry<E> {
    method: Method,
    path: String,
    handler: BoxedHandler<E>,
}

/// Middleware registration entry
pub(crate) struct MiddlewareEntry<E> {
    phase: MiddlewarePhase,
    middleware: Arc<dyn Middleware<E>>,
    options: Option<MiddlewareOptions>,
}

impl<E> Registry<E>
where
    E: Send + Sync + 'static,
{
    /// Create new registry
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = Registry::new();
    /// ```
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Register a route with handler
    ///
    /// Routes are stored in registration order and will be added to the
    /// Dispatcher when `build()` is called.
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method (GET, POST, etc.)
    /// - `path`: Route pattern (e.g., `/users/:id`)
    /// - `handler`: Handler for this route
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// registry.register_route(Method::GET, "/users/:id", handler);
    /// ```
    pub fn register_route(
        &mut self,
        method: Method,
        path: impl Into<String>,
        handler: impl Handler<E>,
    ) {
        self.routes.push(RouteEntry {
            method,
            path: path.into(),
            handler: BoxedHandler::new(handler),
        });
    }

    /// Register middleware with phase and optional filters
    ///
    /// Middleware are stored in registration order and will be added to the
    /// Dispatcher when `build()` is called.
    ///
    /// # Arguments
    ///
    /// - `phase`: When the middleware should run (PreRouting or PostRouting)
    /// - `middleware`: Middleware to register
    /// - `options`: Optional filters (method, pattern, or both)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Global middleware
    /// registry.register_middleware(MiddlewarePhase::PreRouting, logger, None);
    ///
    /// // Path-specific middleware
    /// let opts = MiddlewareOptions::new().on("/api/*");
    /// registry.register_middleware(MiddlewarePhase::PostRouting, auth, Some(opts));
    /// ```
    pub fn register_middleware(
        &mut self,
        phase: MiddlewarePhase,
        middleware: impl Middleware<E>,
        options: Option<MiddlewareOptions>,
    ) {
        self.middleware.push(MiddlewareEntry {
            phase,
            middleware: Arc::new(middleware),
            options,
        });
    }

    /// Consume the registry and return its parts for building a Dispatcher
    ///
    /// This is used internally by App to build the Dispatcher.
    pub(crate) fn into_parts(self) -> (Vec<RouteEntry<E>>, Vec<MiddlewareEntry<E>>) {
        (self.routes, self.middleware)
    }
}

impl<E> Default for Registry<E>
where
    E: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

// Make RouteEntry and MiddlewareEntry accessible to App module
impl<E> RouteEntry<E> {
    pub(crate) fn method(&self) -> &Method {
        &self.method
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn handler(self) -> BoxedHandler<E> {
        self.handler
    }
}

impl<E> MiddlewareEntry<E> {
    pub(crate) fn phase(&self) -> MiddlewarePhase {
        self.phase
    }

    pub(crate) fn middleware(&self) -> Arc<dyn Middleware<E>> {
        Arc::clone(&self.middleware)
    }

    pub(crate) fn options(&self) -> Option<&MiddlewareOptions> {
        self.options.as_ref()
    }
}
