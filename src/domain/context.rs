//! Context - Request/response context

use crate::domain::{Extensions, Request, Response};
use std::sync::Arc;

/// Context - Request carrier with environment and extensions
///
/// Generic over environment type `E` for dependency injection.
pub struct Context<E = ()> {
    /// Request
    pub req: Request,

    /// Environment/bindings
    pub env: Arc<E>,

    /// Type-safe extension bag
    extensions: Extensions,
}

impl<E> Context<E> {
    /// Create a new context
    pub fn new(req: Request, env: Arc<E>) -> Self {
        Self {
            req,
            env,
            extensions: Extensions::new(),
        }
    }

    /// Get typed value from extensions
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    /// Set typed value in extensions
    pub fn set<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.set(value);
    }

    // Response helpers (Hono-style)

    /// Return text response
    pub fn text(&self, text: impl Into<String>) -> Response {
        Response::text(text)
    }

    /// Return JSON response
    pub fn json<T: serde::Serialize>(&self, value: &T) -> Response {
        Response::json(value)
    }

    /// Return HTML response
    pub fn html(&self, html: impl Into<String>) -> Response {
        Response::html(html)
    }

    /// Return not found response
    pub fn not_found(&self) -> Response {
        Response::text("Not Found").with_status(404)
    }
}

impl<E> Clone for Context<E> {
    fn clone(&self) -> Self {
        Self {
            req: self.req.clone(),
            env: Arc::clone(&self.env),
            extensions: self.extensions.clone(),
        }
    }
}
