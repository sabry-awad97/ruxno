//! Middleware trait - Core abstraction for request/response processing
//!
//! Middleware provides a way to intercept and modify requests and responses
//! in a composable chain. Each middleware can:
//! - Inspect or modify the request before passing to the next handler
//! - Inspect or modify the response after the next handler completes
//! - Short-circuit the chain by returning early
//! - Add cross-cutting concerns (logging, auth, CORS, etc.)
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::{Context, Response, Middleware, Next, CoreError};
//!
//! // Simple logging middleware
//! let logger = async |ctx: Context, next: Next| {
//!     println!("Request: {} {}", ctx.req.method(), ctx.req.path());
//!     let response = next.run(ctx).await?;
//!     println!("Response: {}", response.status);
//!     Ok(response)
//! };
//!
//! // Auth middleware that short-circuits
//! let auth = async |ctx: Context, next: Next| {
//!     if ctx.req.header("Authorization").is_none() {
//!         return Ok(Response::unauthorized());
//!     }
//!     next.run(ctx).await
//! };
//! ```

use crate::core::{BoxedHandler, CoreError};
use crate::domain::{Context, Response};
use async_trait::async_trait;
use std::future::Future;

/// Middleware trait for processing requests in a chain
///
/// Middleware sits between the request and the handler, allowing you to:
/// - Modify requests before they reach handlers
/// - Modify responses before they're sent to clients
/// - Short-circuit the chain (e.g., for auth failures)
/// - Add cross-cutting concerns (logging, metrics, etc.)
///
/// # Generic Parameters
///
/// - `E`: Environment type for dependency injection (defaults to `()`)
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::{Context, Response, Middleware, Next, CoreError};
///
/// // Async function middleware
/// let middleware = async |ctx: Context, next: Next| {
///     // Before handler
///     println!("Before: {}", ctx.req.path());
///     
///     // Call next handler
///     let response = next.run(ctx).await?;
///     
///     // After handler
///     println!("After: {}", response.status);
///     Ok(response)
/// };
///
/// // Struct-based middleware
/// struct LoggerMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for LoggerMiddleware {
///     async fn process(&self, ctx: Context, next: Next) -> Result<Response, CoreError> {
///         println!("Request: {}", ctx.req.path());
///         next.run(ctx).await
///     }
/// }
/// ```
#[async_trait]
pub trait Middleware<E = ()>: Send + Sync + 'static {
    /// Process a request with access to the next handler in the chain
    ///
    /// # Arguments
    ///
    /// - `ctx`: The request context
    /// - `next`: The next handler/middleware in the chain
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Successful response (from this middleware or downstream)
    /// - `Err(CoreError)`: Error that will be converted to an error response
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno::{Context, Response, Middleware, Next, CoreError};
    ///
    /// struct TimingMiddleware;
    ///
    /// #[async_trait::async_trait]
    /// impl Middleware for TimingMiddleware {
    ///     async fn process(&self, ctx: Context, next: Next) -> Result<Response, CoreError> {
    ///         let start = std::time::Instant::now();
    ///         let response = next.run(ctx).await?;
    ///         let duration = start.elapsed();
    ///         println!("Request took: {:?}", duration);
    ///         Ok(response)
    ///     }
    /// }
    /// ```
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError>;
}

/// Next handler in the middleware chain
///
/// `Next` represents the continuation of the middleware chain. Calling `next.run(ctx)`
/// passes control to the next middleware or the final handler.
///
/// # Cloning
///
/// `Next` is cheaply cloneable (it wraps an `Arc`), allowing middleware to:
/// - Call the next handler multiple times (e.g., retry logic)
/// - Store the next handler for later execution
/// - Pass the next handler to spawned tasks
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::{Context, Next, Response, CoreError};
///
/// // Retry middleware
/// async fn retry_middleware(ctx: Context, next: Next) -> Result<Response, CoreError> {
///     for attempt in 1..=3 {
///         match next.clone().run(ctx.clone()).await {
///             Ok(response) => return Ok(response),
///             Err(e) if attempt < 3 => {
///                 println!("Attempt {} failed, retrying...", attempt);
///                 continue;
///             }
///             Err(e) => return Err(e),
///         }
///     }
///     unreachable!()
/// }
/// ```
pub struct Next<E = ()> {
    handler: BoxedHandler<E>,
}

impl<E> Clone for Next<E> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }
}

impl<E> Next<E>
where
    E: Send + Sync + 'static,
{
    /// Create a new Next from a boxed handler
    ///
    /// This is typically used internally by the framework when building
    /// middleware chains. Users rarely need to call this directly.
    ///
    /// # Arguments
    ///
    /// - `handler`: The next handler in the chain (wrapped in Arc)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Internal usage in middleware chain builder
    /// let next = Next::new(boxed_handler);
    /// ```
    pub fn new(handler: BoxedHandler<E>) -> Self {
        Self { handler }
    }

    /// Get the underlying handler
    ///
    /// Extracts the boxed handler from Next. Useful for advanced use cases
    /// like Tower adapters or custom middleware chain builders.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let handler = next.into_handler();
    /// // Use handler directly
    /// ```
    pub fn into_handler(self) -> BoxedHandler<E> {
        self.handler
    }

    /// Run the next handler in the chain
    ///
    /// This consumes `self` and passes the context to the next handler.
    /// If you need to call the next handler multiple times, clone `Next` first.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The request context to pass to the next handler
    ///
    /// # Returns
    ///
    /// - `Ok(Response)`: Response from the next handler or downstream
    /// - `Err(CoreError)`: Error from the next handler or downstream
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ruxno::{Context, Next, Response, CoreError};
    ///
    /// async fn simple_middleware(ctx: Context, next: Next) -> Result<Response, CoreError> {
    ///     // Just pass through to next handler
    ///     next.run(ctx).await
    /// }
    ///
    /// async fn modifying_middleware(mut ctx: Context, next: Next) -> Result<Response, CoreError> {
    ///     // Modify context before passing to next
    ///     ctx.set("custom-header", "value");
    ///     next.run(ctx).await
    /// }
    /// ```
    pub async fn run(self, ctx: Context<E>) -> Result<Response, CoreError> {
        self.handler.handle(ctx).await
    }
}

// Implement Middleware for async closures
//
// This implementation allows using closures directly as middleware:
//
// ```rust
// app.use_middleware("*", async |ctx: Context, next: Next| {
//     println!("Before handler");
//     let response = next.run(ctx).await?;
//     println!("After handler");
//     Ok(response)
// });
// ```
//
// The closure must return a Future that resolves to Result<Response, CoreError>.
#[async_trait]
impl<E, F, Fut> Middleware<E> for F
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>, Next<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        self(ctx, next).await
    }
}

/// Implement From trait for Next from async closures
///
/// This trait implementation enables ergonomic conversion from async handler
/// closures to `Next` instances, which represent the next handler in a
/// middleware chain.
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
/// use ruxno::{Context, Next, Response, CoreError};
///
/// // Create Next from a handler closure
/// let next: Next = (async |ctx: Context| {
///     Ok(ctx.text("Success"))
/// }).into();
///
/// // With environment type
/// struct MyEnv { db: String }
/// let next: Next<MyEnv> = (async |ctx: Context<MyEnv>| {
///     Ok(ctx.text("Success"))
/// }).into();
///
/// // Use in middleware (automatic conversion)
/// async fn my_middleware(ctx: Context, next: Next) -> Result<Response, CoreError> {
///     println!("Before handler");
///     let response = next.run(ctx).await?;
///     println!("After handler");
///     Ok(response)
/// }
/// ```
///
/// # Usage in Middleware Chains
///
/// This conversion is used internally when building middleware chains:
///
/// ```rust,ignore
/// // Framework code (internal)
/// let final_handler = async |ctx: Context| {
///     Ok(Response::text("Final handler"))
/// };
/// let next: Next = final_handler.into();
/// middleware.process(ctx, next).await
/// ```
///
/// # Note
///
/// We implement `From` for closures specifically (not for all `Handler` types)
/// to avoid conflicts with the blanket `impl<T> From<T> for T` in std.
impl<E, F, Fut> From<F> for Next<E>
where
    E: Send + Sync + 'static,
    F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, CoreError>> + Send + 'static,
{
    fn from(handler: F) -> Self {
        Next::new(BoxedHandler::new(handler))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_next_cloning() {
        // Create a simple handler
        let handler = BoxedHandler::new(|_ctx: Context| async move { Ok(Response::default()) });
        let next = Next::new(handler);

        // Verify Next is cheaply cloneable
        let cloned = next.clone();

        // Both should work
        let _next1 = next;
        let _next2 = cloned;
    }

    #[tokio::test]
    async fn test_middleware_trait_object() {
        // Verify we can create trait objects
        let _middleware: Arc<dyn Middleware> =
            Arc::new(|_ctx: Context, next: Next| async move { next.run(_ctx).await });

        // This compiles, which proves Middleware is dyn-compatible
    }

    #[tokio::test]
    async fn test_middleware_arc_creation() {
        // Create middleware using Arc::new directly
        let middleware: Arc<dyn Middleware> =
            Arc::new(|_ctx: Context, next: Next| async move { next.run(_ctx).await });

        // Verify it's a trait object (Arc)
        assert_eq!(Arc::strong_count(&middleware), 1);

        // Clone it cheaply
        let _cloned = Arc::clone(&middleware);
        assert_eq!(Arc::strong_count(&middleware), 2);
    }

    #[tokio::test]
    async fn test_middleware_with_environment() {
        // Define a custom environment type
        struct MyEnv {
            _value: String,
        }

        // Create middleware with environment
        let middleware: Arc<dyn Middleware<MyEnv>> =
            Arc::new(|_ctx: Context<MyEnv>, next: Next<MyEnv>| async move { next.run(_ctx).await });

        // Verify type inference works
        let _: Arc<dyn Middleware<MyEnv>> = middleware;
    }

    #[tokio::test]
    async fn test_middleware_collection() {
        // Store multiple middleware in a collection
        let mut middlewares: Vec<Arc<dyn Middleware>> = vec![];

        middlewares.push(Arc::new(|_ctx: Context, next: Next| async move {
            next.run(_ctx).await
        }));

        middlewares.push(Arc::new(async |_ctx: Context, next: Next| {
            next.run(_ctx).await
        }));

        // Verify we can store different closures in the same collection
        assert_eq!(middlewares.len(), 2);
    }

    #[tokio::test]
    async fn test_next_from_handler() {
        // Test From trait implementation for Next
        let handler = async |_ctx: Context| Ok(Response::default());

        // Use .into() to convert handler to Next
        let next: Next = handler.into();

        // Verify it's cloneable
        let _cloned = next.clone();
    }

    #[tokio::test]
    async fn test_next_from_handler_with_environment() {
        // Define a custom environment type
        struct MyEnv {
            _value: String,
        }

        // Use From trait with environment
        let handler = async |_ctx: Context<MyEnv>| Ok(Response::default());
        let next: Next<MyEnv> = handler.into();

        // Verify type inference works
        let _: Next<MyEnv> = next;
    }

    // Note: Full integration tests will be added once Context and Response
    // are fully implemented
}
