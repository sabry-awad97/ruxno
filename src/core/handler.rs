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
//! // Async function handler
//! let handler = async |ctx: Context| {
//!     Ok(ctx.text("Hello, World!"))
//! };
//!
//! // Handler with error handling
//! let handler = async |ctx: Context| {
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
/// // Async function handler
/// let handler = async |ctx: Context| {
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
/// This newtype wraps `Arc<dyn Handler<E>>` to provide:
/// - **Type Erasure**: Allows storing different handler types in the same collection
/// - **Zero-Cost Cloning**: Arc cloning is just incrementing a reference count
/// - **Thread Safety**: Arc ensures handlers can be shared across threads safely
///
/// # For End Users
///
/// You don't create `BoxedHandler` directly. Instead, use the ergonomic closure syntax:
///
/// ```rust,no_run
/// use ruxno::App;
///
/// let mut app = App::new();
///
/// // The framework automatically converts closures to BoxedHandler
/// app.get("/", async |ctx| {
///     Ok(ctx.text("Hello!"))
/// });
/// ```
///
/// # For Framework Internals
///
/// The framework uses `BoxedHandler::new()` internally when building routing tables.
/// The `From` trait implementation allows automatic conversion from closures.
pub struct BoxedHandler<E = ()>(Arc<dyn Handler<E>>);

// Manual Clone implementation since Arc<T> is Clone regardless of T's bounds
impl<E> Clone for BoxedHandler<E> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<E> BoxedHandler<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new BoxedHandler from any type implementing Handler (internal use only)
    ///
    /// This is used internally by the framework when building routing tables.
    /// End users should use closure syntax instead:
    ///
    /// ```rust,no_run
    /// app.get("/", async |ctx: Context| {
    ///     Ok(ctx.text("Hello!"))
    /// });
    /// ```
    pub(crate) fn new(handler: impl Handler<E>) -> Self {
        Self(Arc::new(handler))
    }

    /// Get a reference to the inner Arc (internal use only)
    ///
    /// Used internally for checking reference counts in tests.
    #[cfg(test)]
    pub(crate) fn inner(&self) -> &Arc<dyn Handler<E>> {
        &self.0
    }

    /// Handle a request using the wrapped handler
    ///
    /// This is called internally by the dispatcher to execute handlers.
    pub(crate) async fn handle(&self, ctx: Context<E>) -> Result<Response, CoreError> {
        self.0.handle(ctx).await
    }
}

/// Implement From trait for BoxedHandler from async closures
///
/// This trait implementation enables automatic conversion from async closures
/// to `BoxedHandler`, which is used internally by the routing system when
/// users register routes.
///
/// # Type Parameters
///
/// - `E`: Environment type for dependency injection (defaults to `()`)
/// - `F`: Closure type that takes `Context<E>` and returns a Future
/// - `Fut`: Future type that resolves to `Result<Response, CoreError>`
///
/// # Trait Bounds
///
/// - `F: Fn(Context<E>) -> Fut` - Closure must be callable multiple times
/// - `F: Send + Sync + 'static` - Closure must be thread-safe and have static lifetime
/// - `Fut: Future<Output = Result<Response, CoreError>>` - Must return Result
/// - `Fut: Send + 'static` - Future must be sendable across threads
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::{BoxedHandler, Context, Response, CoreError};
///
/// // Automatic conversion when registering routes (internal use)
/// let handler: BoxedHandler = (async |ctx: Context| {
///     Ok(ctx.text("Hello!"))
/// }).into();
///
/// // With environment type
/// struct MyEnv { db: String }
/// let handler: BoxedHandler<MyEnv> = (async |ctx: Context<MyEnv>| {
///     Ok(ctx.text("Hello from env!"))
/// }).into();
/// ```
///
/// # For End Users
///
/// You don't need to call `.into()` explicitly. The framework automatically
/// converts closures when you register routes:
///
/// ```rust,no_run
/// app.get("/", async |ctx| {
///     Ok(ctx.text("Hello!"))
/// });
/// ```
impl<E, F, Fut> From<F> for BoxedHandler<E>
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    fn from(handler: F) -> Self {
        Self(Arc::new(handler))
    }
}

// Implement Handler for async closures
//
// This implementation allows using async closures directly as handlers:
//
// ```rust
// app.get("/", async |ctx: Context| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_boxed_handler_new() {
        // Create handler using BoxedHandler::new
        let handler = BoxedHandler::new(async |_ctx: Context| Ok(Response::default()));

        // Verify it's a BoxedHandler (Arc)
        assert_eq!(Arc::strong_count(handler.inner()), 1);

        // Clone it cheaply
        let cloned = handler.clone();
        assert_eq!(Arc::strong_count(handler.inner()), 2);
        assert_eq!(Arc::strong_count(cloned.inner()), 2);
    }

    #[tokio::test]
    async fn test_boxed_handler_with_environment() {
        // Define a custom environment type
        struct MyEnv {
            _value: String,
        }

        // Create handler with environment
        let handler = BoxedHandler::new(async |_ctx: Context<MyEnv>| Ok(Response::default()));

        // Verify type inference works
        let _: BoxedHandler<MyEnv> = handler;
    }

    #[tokio::test]
    async fn test_boxed_handler_collection() {
        // Store multiple handlers in a collection
        let mut handlers: Vec<BoxedHandler> = vec![];

        handlers.push(BoxedHandler::new(async |_ctx: Context| {
            Ok(Response::default())
        }));

        handlers.push(BoxedHandler::new(async |_ctx: Context| {
            Ok(Response::default())
        }));

        // Verify we can store different closures in the same collection
        assert_eq!(handlers.len(), 2);
    }

    // Note: Full integration tests will be added once Context and Response
    // are fully implemented
}
