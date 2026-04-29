//! Route and middleware registry

use crate::core::{BoxedHandler, Method, Middleware};
use std::sync::Arc;

/// Registry for routes and middleware
pub struct Registry<E = ()> {
    routes: Vec<RouteEntry<E>>,
    middleware: Vec<Arc<dyn Middleware<E>>>,
}

struct RouteEntry<E> {
    method: Method,
    path: String,
    handler: BoxedHandler<E>,
}

impl<E> Registry<E>
where
    E: Send + Sync + 'static,
{
    /// Create new registry
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Register route
    pub fn register_route(&mut self, method: Method, path: String, handler: BoxedHandler<E>) {
        self.routes.push(RouteEntry {
            method,
            path,
            handler,
        });
    }

    /// Register middleware
    pub fn register_middleware(&mut self, middleware: Arc<dyn Middleware<E>>) {
        self.middleware.push(middleware);
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
