//! # Ruxno Middleware
//!
//! Optional middleware collection for the Ruxno web framework.
//!
//! This crate provides production-ready middleware that can be easily integrated
//! into Ruxno applications. Each middleware is feature-gated, so you only pay
//! for what you use.
//!
//! ## Features
//!
//! - **rate-limit**: In-memory rate limiting using `governor`
//! - **rate-limit-redis**: Distributed rate limiting with Redis
//! - **cors**: Cross-Origin Resource Sharing (CORS) support
//! - **compression**: Response compression (gzip, brotli)
//! - **auth**: JWT authentication helpers
//! - **logger**: Request/response logging
//! - **security-headers**: Security headers (HSTS, CSP, etc.)
//! - **pretty-json**: Pretty-print JSON responses with configurable indentation
//!
//! ## Usage
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ruxno = "0.1"
//! ruxno-middleware = { version = "0.1", features = ["rate-limit", "cors"] }
//! ```
//!
//! ## Examples
//!
//! ### Rate Limiting
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::RateLimitMiddleware;
//!
//! let mut app = App::new();
//!
//! // 100 requests per second per IP
//! let rate_limiter = RateLimitMiddleware::per_second(100);
//! app.use_middleware("*", rate_limiter.middleware());
//! ```
//!
//! ### CORS
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::CorsMiddleware;
//!
//! let mut app = App::new();
//!
//! let cors = CorsMiddleware::new()
//!     .allow_origin("https://example.com")
//!     .allow_methods(vec!["GET", "POST"])
//!     .allow_credentials(true);
//!
//! app.use_middleware("*", cors.middleware());
//! ```
//!
//! ### Pretty JSON
//!
//! ```rust,ignore
//! use ruxno::App;
//! use ruxno_middleware::pretty_json;
//!
//! let mut app = App::new();
//!
//! // Simple usage with default settings (2-space indentation)
//! app.r#use(pretty_json());
//!
//! // Or with custom configuration
//! use ruxno_middleware::PrettyJsonMiddleware;
//! app.r#use(PrettyJsonMiddleware::with_indent(4));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export ruxno types for convenience
pub use ruxno::core::{CoreError, Next};
pub use ruxno::{Context, Response};

// Middleware modules (feature-gated)
#[cfg(feature = "rate-limit")]
pub mod rate_limit;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "compression")]
pub mod compression;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "logger")]
pub mod logger;

#[cfg(feature = "security-headers")]
pub mod security_headers;

#[cfg(feature = "pretty-json")]
pub mod pretty_json;

// Re-exports for convenience
#[cfg(feature = "rate-limit")]
pub use rate_limit::RateLimitMiddleware;

#[cfg(feature = "cors")]
pub use cors::CorsMiddleware;

#[cfg(feature = "compression")]
pub use compression::CompressionMiddleware;

#[cfg(feature = "auth")]
pub use auth::AuthMiddleware;

#[cfg(feature = "logger")]
pub use logger::LoggerMiddleware;

#[cfg(feature = "security-headers")]
pub use security_headers::SecurityHeadersMiddleware;

#[cfg(feature = "pretty-json")]
pub use pretty_json::{pretty_json, PrettyJsonMiddleware};
