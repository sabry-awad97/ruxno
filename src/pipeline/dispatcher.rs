//! Dispatcher - Unified routing and middleware dispatch

use crate::core::{BoxedHandler, CoreError, Handler, Method, Middleware};
use crate::domain::{Context, Request, Response};
use crate::routing::Router;
use std::sync::Arc;

/// Dispatcher - Orchestrates request handling
pub struct Dispatcher<E = ()> {
    /// Router for route matching
    router: Router<E>,

    /// Global middleware
    global_middleware: Vec<Arc<dyn Middleware<E>>>,

    /// Environment
    env: Arc<E>,
}

impl<E> Dispatcher<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new dispatcher
    pub fn new(env: Arc<E>) -> Self {
        Self {
            router: Router::new(),
            global_middleware: Vec::new(),
            env,
        }
    }

    /// Register a route
    pub fn register_route(
        &mut self,
        method: Method,
        path: &str,
        handler: impl Handler<E>,
    ) -> Result<(), CoreError> {
        // TODO: Wrap handler with applicable middleware
        // TODO: Insert into router
        todo!("Implement Dispatcher::register_route")
    }

    /// Register global middleware
    pub fn register_middleware(&mut self, middleware: impl Middleware<E>) {
        self.global_middleware.push(Arc::new(middleware));
    }

    /// Dispatch a request
    pub async fn dispatch(&self, mut req: Request) -> Result<Response, CoreError> {
        // TODO: Lookup route
        // TODO: Extract parameters
        // TODO: Create context
        // TODO: Execute handler with middleware
        todo!("Implement Dispatcher::dispatch")
    }
}
