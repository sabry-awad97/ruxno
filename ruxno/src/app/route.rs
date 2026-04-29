//! Route builder for chaining multiple methods on the same path

use crate::app::App;
use crate::core::{Handler, Method, Middleware};

/// Route builder for chaining HTTP methods on the same path
///
/// Allows syntax like: `app.route("/users").get(handler).post(handler)`
pub struct Route<'a, E = ()> {
    app: &'a mut App<E>,
    path: String,
}

impl<'a, E> Route<'a, E>
where
    E: Send + Sync + 'static,
{
    /// Create a new route builder
    pub(crate) fn new(app: &'a mut App<E>, path: impl Into<String>) -> Self {
        Self {
            app,
            path: path.into(),
        }
    }

    /// Register middleware for this route path
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ruxno_clean::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.route("/api")
    ///     .r#use(async |ctx: Context, next: Next| {
    ///         // Auth middleware
    ///         next.run(ctx).await
    ///     })
    ///     .get(async |c: Context| c.text("API endpoint"));
    /// ```
    pub fn r#use(self, middleware: impl Middleware<E>) -> Self {
        self.app.r#use(middleware);
        self
    }

    /// Register GET handler
    pub fn get(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::GET, &self.path, handler);
        self
    }

    /// Register POST handler
    pub fn post(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::POST, &self.path, handler);
        self
    }

    /// Register PUT handler
    pub fn put(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::PUT, &self.path, handler);
        self
    }

    /// Register DELETE handler
    pub fn delete(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::DELETE, &self.path, handler);
        self
    }

    /// Register PATCH handler
    pub fn patch(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::PATCH, &self.path, handler);
        self
    }

    /// Register OPTIONS handler
    pub fn options(self, handler: impl Handler<E>) -> Self {
        self.app
            .register_route(Method::OPTIONS, &self.path, handler);
        self
    }

    /// Register HEAD handler
    pub fn head(self, handler: impl Handler<E>) -> Self {
        self.app.register_route(Method::HEAD, &self.path, handler);
        self
    }
}
