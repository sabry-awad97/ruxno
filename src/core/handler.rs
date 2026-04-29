//! Handler trait - Core abstraction for request handlers
//!
//! This module defines the `Handler` trait, which is the fundamental abstraction
//! for processing HTTP requests. Handlers are async functions that take a `Context`
//! and return a `Response`.
//!
//! # Implementation Note
//!
//! Currently uses `async_trait` for trait object compatibility. Once Rust's native
//! async traits support dyn compatibility (expected in future Rust versions), this
//! will be migrated to use native `async fn` in traits.
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::{Context, Response, Handler, CoreError};
//!
//! // Async closure handler
//! let handler = |ctx: Context| async move {
//!     Ok(ctx.text("Hello, World!"))
//! };
//!
//! // Handler with error handling
//! let handler = |ctx: Context| async move {
//!     let id = ctx.req.param("id")?;
//!     Ok(ctx.json(&serde_json::json!({ "id": id })))
//! };
//! ```

use crate::core::CoreError;
use crate::domain::{Context, Response};
use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

/// Handler trait for processing requests
///
/// This trait is automatically implemented for async closures that match the signature:
/// `Fn(Context<E>) -> Future<Output = Result<Response, CoreError>>`
///
/// # Generic Parameters
///
/// - `E`: Environment type for dependency injection (defaults to `()`)
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::{Context, Response, Handler, CoreError};
///
/// // Async closure handler
/// let handler = async |ctx: Context| move {
///     Ok(ctx.text("Hello!"))
/// };
///
/// // Struct-based handler
/// struct MyHandler;
///
/// #[async_trait::async_trait]
/// impl Handler for MyHandler {
///     async fn handle(&self, ctx: Context) -> Result<Response, CoreError> {
///         Ok(ctx.text("Hello from struct!"))
///     }
/// }
/// ```
#[async_trait]
pub trait Handler<E = ()>: Send + Sync + 'static {
    /// Handle a request and return a response
    ///
    /// # Arguments
    ///
    /// - `ctx`: The request context containing the request, environment, and extensions
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Successful response
    /// - `Err(CoreError)`: Error that will be converted to an error response
    async fn handle(&self, ctx: Context<E>) -> Result<Response, CoreError>;
}

/// Type-erased handler wrapped in Arc for efficient cloning
///
/// This type alias is used internally to store handlers in the routing table.
/// The `Arc` allows handlers to be cloned cheaply without duplicating the
/// underlying handler implementation.
///
/// # Why Arc<dyn Handler>?
///
/// - **Type Erasure**: Allows storing different handler types in the same collection
/// - **Zero-Cost Cloning**: Arc cloning is just incrementing a reference count
/// - **Thread Safety**: Arc ensures handlers can be shared across threads safely
pub type BoxedHandler<E = ()> = Arc<dyn Handler<E>>;

// Implement Handler for async closures
//
// This implementation allows using async closures directly as handlers:
//
// ```rust
// app.get("/", async |ctx: Context| move {
//     Ok(ctx.text("Hello!"))
// });
// ```
//
// The closure must return a Future that resolves to Result<Response, CoreError>.
#[async_trait]
impl<E, F, Fut> Handler<E> for F
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    async fn handle(&self, ctx: Context<E>) -> Result<Response, CoreError> {
        self(ctx).await
    }
}

/// Create a boxed handler from an async closure (internal utility)
///
/// This is an internal utility function used by the framework to convert
/// async closures into `BoxedHandler` instances. It's used internally by
/// the routing and middleware systems.
///
/// # Type Parameters
///
/// - `E`: Environment type (defaults to `()`)
/// - `F`: Closure type that implements `Handler<E>`
///
/// # Internal Usage
///
/// ```rust,ignore
/// // Used internally when registering routes
/// let handler = make_handler(|ctx: Context| async move {
///     Ok(ctx.text("Hello!"))
/// });
/// router.insert(Method::GET, "/", handler)?;
/// ```
///
/// # Note for Users
///
/// You don't need to call this function directly. The framework automatically
/// converts closures to handlers when you register routes:
///
/// ```rust,no_run
/// app.get("/", async |ctx: Context| move {
///     Ok(ctx.text("Hello!"))
/// });
/// ```
pub(crate) fn make_handler<E, F, Fut>(handler: F) -> BoxedHandler<E>
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    Arc::new(handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_make_handler() {
        // Create handler using make_handler utility
        let handler = make_handler(async |_ctx: Context| Ok(Response::default()));

        // Verify it's a BoxedHandler (Arc)
        assert_eq!(Arc::strong_count(&handler), 1);

        // Clone it cheaply
        let _cloned = Arc::clone(&handler);
        assert_eq!(Arc::strong_count(&handler), 2);
    }

    #[tokio::test]
    async fn test_make_handler_with_environment() {
        // Define a custom environment type
        struct MyEnv {
            _value: String,
        }

        // Create handler with environment
        let handler = make_handler(async |_ctx: Context<MyEnv>| Ok(Response::default()));

        // Verify type inference works
        let _: BoxedHandler<MyEnv> = handler;
    }

    #[tokio::test]
    async fn test_make_handler_collection() {
        // Store multiple handlers in a collection
        let mut handlers: Vec<BoxedHandler> = vec![];

        handlers.push(make_handler(async |_ctx: Context| Ok(Response::default())));

        handlers.push(make_handler(async |_ctx: Context| Ok(Response::default())));

        // Verify we can store different closures in the same collection
        assert_eq!(handlers.len(), 2);
    }

    // Note: Full integration tests will be added once Context and Response
    // are fully implemented
}
