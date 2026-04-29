//! Middleware chain builder

use crate::core::{BoxedHandler, Middleware};
use std::sync::Arc;

/// Middleware chain
pub struct MiddlewareChain<E = ()> {
    /// Middleware stack
    middleware: Vec<Arc<dyn Middleware<E>>>,

    /// Final handler
    handler: BoxedHandler<E>,
}

impl<E> MiddlewareChain<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new chain
    pub fn new(handler: BoxedHandler<E>) -> Self {
        Self {
            middleware: Vec::new(),
            handler,
        }
    }

    /// Add middleware to the chain
    pub fn add(&mut self, middleware: Arc<dyn Middleware<E>>) {
        self.middleware.push(middleware);
    }

    /// Build the pre-computed onion
    pub fn build(self) -> BoxedHandler<E> {
        // TODO: Fold middleware into nested closures
        // TODO: Return wrapped handler
        todo!("Implement MiddlewareChain::build")
    }
}
