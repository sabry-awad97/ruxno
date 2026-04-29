//! Middleware trait - Core abstraction for middleware

use crate::core::{BoxedHandler, CoreError};
use crate::domain::Context;
use crate::domain::Response;

/// Middleware trait
///
/// Middleware can inspect and modify requests/responses, short-circuit
/// the chain, or pass control to the next handler.
#[async_trait::async_trait]
pub trait Middleware<E = ()>: Send + Sync + 'static {
    /// Process a request with access to the next handler
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError>;
}

/// Next handler in the middleware chain
#[derive(Clone)]
pub struct Next<E = ()> {
    handler: BoxedHandler<E>,
}

impl<E> Next<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new Next from a boxed handler
    pub fn new(handler: BoxedHandler<E>) -> Self {
        Self { handler }
    }

    /// Run the next handler
    pub async fn run(self, ctx: Context<E>) -> Result<Response, CoreError> {
        self.handler.handle(ctx).await
    }
}

// Implement Middleware for async closures
#[async_trait::async_trait]
impl<E, F, Fut> Middleware<E> for F
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>, Next<E>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        self(ctx, next).await
    }
}
