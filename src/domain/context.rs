//! Context - Request/response context
//!
//! This module defines the `Context` type, which is the main carrier for
//! request/response handling. It provides ergonomic access to the request,
//! environment (for DI), and extensions (for middleware data).
//!
//! # Design
//!
//! - **Generic environment**: `Context<E>` allows dependency injection
//! - **Immutable request**: Request is Arc-wrapped for cheap cloning
//! - **Type-safe extensions**: Store arbitrary typed data for middleware
//! - **Convenience methods**: Hono-style `text()`, `json()`, `html()` helpers
//!
//! # Examples
//!
//! ```rust,no_run
//! use ruxno::{Context, CoreError, Response};
//!
//! async fn handler(ctx: Context) -> Result<Response, CoreError> {
//!     // Access request
//!     let id = ctx.req.param("id")?;
//!     
//!     // Return JSON response
//!     Ok(ctx.json(&serde_json::json!({ "id": id })))
//! }
//!
//! // With environment
//! struct Env {
//!     db: Database,
//! }
//!
//! async fn handler_with_env(ctx: Context<Env>) -> Result<Response, CoreError> {
//!     let user = ctx.env().db.get_user(123).await?;
//!     Ok(ctx.json(&user))
//! }
//! ```

use crate::core::StatusCode;
use crate::domain::{Extensions, Request, Response};
use std::sync::Arc;

/// Context - Request carrier with environment and extensions
///
/// The main context type that handlers receive. Provides ergonomic access
/// to the request, environment (for dependency injection), and extensions
/// (for middleware data).
///
/// # Generic Parameter
///
/// - `E`: Environment type for dependency injection (defaults to `()`)
///
/// # Examples
///
/// ```rust,no_run
/// use ruxno::{Context, CoreError, Response};
///
/// // Simple handler (no environment)
/// async fn hello(ctx: Context) -> Result<Response, CoreError> {
///     Ok(ctx.text("Hello, World!"))
/// }
///
/// // Handler with environment
/// struct AppEnv {
///     db: Database,
///     config: Config,
/// }
///
/// async fn get_user(ctx: Context<AppEnv>) -> Result<Response, CoreError> {
///     let id = ctx.req.param("id")?;
///     let user = ctx.env().db.get_user(id).await?;
///     Ok(ctx.json(&user))
/// }
/// ```
pub struct Context<E = ()> {
    /// Request accessor
    ///
    /// Provides access to HTTP method, URI, headers, body, path parameters,
    /// and query parameters.
    pub req: Request,

    /// Environment/bindings (for dependency injection)
    ///
    /// Wrapped in Arc for cheap cloning. Access via `ctx.env()`.
    env: Arc<E>,

    /// Type-safe extension bag for middleware data
    ///
    /// Use `ctx.get::<T>()` and `ctx.set::<T>()` to store/retrieve typed data.
    extensions: Extensions,
}

impl<E> Context<E> {
    /// Create a new context
    ///
    /// # Arguments
    ///
    /// - `req`: HTTP request
    /// - `env`: Environment for dependency injection (wrapped in Arc)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ctx = Context::new(req, Arc::new(env));
    /// ```
    pub fn new(req: Request, env: Arc<E>) -> Self {
        Self {
            req,
            env,
            extensions: Extensions::new(),
        }
    }

    /// Get reference to environment
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let db = &ctx.env().db;
    /// let user = db.get_user(123).await?;
    /// ```
    pub fn env(&self) -> &E {
        &self.env
    }

    /// Get typed value from extensions
    ///
    /// Extensions allow middleware to store typed data that can be
    /// retrieved by handlers or other middleware.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Middleware sets user
    /// ctx.set(User { id: 123, name: "John" });
    ///
    /// // Handler retrieves user
    /// if let Some(user) = ctx.get::<User>() {
    ///     println!("User: {}", user.name);
    /// }
    /// ```
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }

    /// Set typed value in extensions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// ctx.set(RequestId("abc123".to_string()));
    /// ctx.set(User { id: 123, name: "John" });
    /// ```
    pub fn set<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.set(value);
    }

    /// Remove typed value from extensions
    ///
    /// Returns the value if it existed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let user = ctx.remove::<User>();
    /// ```
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.extensions.remove::<T>()
    }

    // ========================================================================
    // Response Helpers (Hono-style)
    // ========================================================================

    /// Return text response
    ///
    /// Creates a response with `Content-Type: text/plain; charset=utf-8`
    /// and 200 OK status.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Ok(ctx.text("Hello, World!"))
    /// ```
    pub fn text(&self, text: impl Into<String>) -> Response {
        Response::text(text)
    }

    /// Return JSON response
    ///
    /// Creates a response with `Content-Type: application/json`
    /// and 200 OK status. Returns 500 if serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Ok(ctx.json(&serde_json::json!({ "status": "ok" })))
    /// ```
    pub fn json<T: serde::Serialize>(&self, value: &T) -> Response {
        Response::json(value)
    }

    /// Return HTML response
    ///
    /// Creates a response with `Content-Type: text/html; charset=utf-8`
    /// and 200 OK status.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Ok(ctx.html("<h1>Hello</h1>"))
    /// ```
    pub fn html(&self, html: impl Into<String>) -> Response {
        Response::html(html)
    }

    /// Return redirect response
    ///
    /// Creates a 302 Found redirect. Use `.with_status()` to change
    /// to 301 (permanent) or 307/308 (preserve method).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Temporary redirect (302)
    /// Ok(ctx.redirect("/login"))
    ///
    /// // Permanent redirect (301)
    /// Ok(ctx.redirect("/new-url").with_status(StatusCode::MOVED_PERMANENTLY))
    /// ```
    pub fn redirect(&self, location: impl Into<String>) -> Response {
        Response::redirect(location)
    }

    /// Return not found response
    ///
    /// Creates a 404 Not Found response with "Not Found" text.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Ok(ctx.not_found())
    /// ```
    pub fn not_found(&self) -> Response {
        Response::text("Not Found").with_status(StatusCode::NOT_FOUND)
    }

    /// Return empty response with status
    ///
    /// Useful for 204 No Content, 201 Created, etc.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // 204 No Content
    /// Ok(ctx.status(StatusCode::NO_CONTENT))
    ///
    /// // 201 Created with location header
    /// Ok(ctx.status(StatusCode::CREATED)
    ///     .with_header("location", "/users/123"))
    /// ```
    pub fn status(&self, status: StatusCode) -> Response {
        Response::new().with_status(status)
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

#[cfg(test)]
mod tests {
    use crate::http::Headers;

    use super::*;
    use bytes::Bytes;
    use http::Uri;
    use std::collections::HashMap;

    fn create_test_context() -> Context<()> {
        let req = Request::new(
            crate::core::Method::GET,
            "/api/users".parse::<Uri>().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::new(),
        );
        Context::new(req, Arc::new(()))
    }

    #[test]
    fn test_context_new() {
        let ctx = create_test_context();
        assert_eq!(ctx.req.path(), "/api/users");
    }

    #[test]
    fn test_context_env() {
        struct TestEnv {
            value: i32,
        }

        let req = Request::new(
            crate::core::Method::GET,
            "/".parse::<Uri>().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::new(),
        );
        let ctx = Context::new(req, Arc::new(TestEnv { value: 42 }));

        assert_eq!(ctx.env().value, 42);
    }

    #[test]
    fn test_context_extensions_set_get() {
        let mut ctx = create_test_context();

        #[derive(Debug, PartialEq)]
        struct UserId(i32);

        ctx.set(UserId(123));
        assert_eq!(ctx.get::<UserId>(), Some(&UserId(123)));
        assert_eq!(ctx.get::<String>(), None);
    }

    #[test]
    fn test_context_extensions_remove() {
        let mut ctx = create_test_context();

        #[derive(Debug, PartialEq)]
        struct UserId(i32);

        ctx.set(UserId(123));
        assert_eq!(ctx.remove::<UserId>(), Some(UserId(123)));
        assert_eq!(ctx.get::<UserId>(), None);
    }

    #[test]
    fn test_context_text() {
        let ctx = create_test_context();
        let res = ctx.text("Hello, World!");

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
    }

    #[test]
    fn test_context_json() {
        let ctx = create_test_context();
        let data = serde_json::json!({ "name": "John", "age": 30 });
        let res = ctx.json(&data);

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_context_html() {
        let ctx = create_test_context();
        let res = ctx.html("<h1>Hello</h1>");

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn test_context_redirect() {
        let ctx = create_test_context();
        let res = ctx.redirect("/login");

        assert_eq!(res.status(), StatusCode::FOUND);
        assert_eq!(res.headers().get("location").unwrap(), "/login");
    }

    #[test]
    fn test_context_not_found() {
        let ctx = create_test_context();
        let res = ctx.not_found();

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_context_status() {
        let ctx = create_test_context();
        let res = ctx.status(StatusCode::NO_CONTENT);

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_context_clone() {
        let ctx = create_test_context();
        let ctx2 = ctx.clone();

        assert_eq!(ctx.req.path(), ctx2.req.path());
        assert_eq!(Arc::strong_count(&ctx.env), 2);
    }

    #[test]
    fn test_context_extensions_multiple_types() {
        let mut ctx = create_test_context();

        #[derive(Debug, PartialEq)]
        struct UserId(i32);

        #[derive(Debug, PartialEq)]
        struct UserName(String);

        ctx.set(UserId(123));
        ctx.set(UserName("John".to_string()));

        assert_eq!(ctx.get::<UserId>(), Some(&UserId(123)));
        assert_eq!(ctx.get::<UserName>(), Some(&UserName("John".to_string())));
    }

    #[test]
    fn test_context_extensions_overwrite() {
        let mut ctx = create_test_context();

        #[derive(Debug, PartialEq)]
        struct Counter(i32);

        ctx.set(Counter(1));
        ctx.set(Counter(2));

        assert_eq!(ctx.get::<Counter>(), Some(&Counter(2)));
    }

    #[test]
    fn test_context_with_environment() {
        struct Database {
            name: String,
        }

        struct AppEnv {
            db: Database,
            api_key: String,
        }

        let env = AppEnv {
            db: Database {
                name: "mydb".to_string(),
            },
            api_key: "secret".to_string(),
        };

        let req = Request::new(
            crate::core::Method::GET,
            "/".parse::<Uri>().unwrap(),
            HashMap::new(),
            Headers::new(),
            Bytes::new(),
        );
        let ctx = Context::new(req, Arc::new(env));

        assert_eq!(ctx.env().db.name, "mydb");
        assert_eq!(ctx.env().api_key, "secret");
    }
}
