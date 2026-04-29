//! Handler trait - Core abstraction for request handlers

use crate::core::CoreError;
use crate::domain::Context;
use crate::domain::Response;
use std::sync::Arc;

/// Handler trait for processing requests
///
/// Generic over environment type `E` for dependency injection.
#[async_trait::async_trait]
pub trait Handler<E = ()>: Send + Sync + 'static {
    /// Handle a request and return a response
    async fn handle(&self, ctx: Context<E>) -> Result<Response, CoreError>;
}

/// Type alias for boxed handlers
pub type BoxedHandler<E = ()> = Arc<dyn Handler<E>>;

// Implement Handler for async closures
#[async_trait::async_trait]
impl<E, F, Fut> Handler<E> for F
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    async fn handle(&self, ctx: Context<E>) -> Result<Response, CoreError> {
        self(ctx).await
    }
}
