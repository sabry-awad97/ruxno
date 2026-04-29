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
//! - **health-check**: Health check endpoints for monitoring and load balancers
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
//! ```
//! use ruxno_middleware::{rate_limit, global_rate_limit, RateLimit, RateLimitMode};
//! use std::time::Duration;
//!
//! // Simple per-IP rate limiting
//! let per_ip_limiter = rate_limit::<()>(100); // 100 requests per second per IP
//!
//! // Global rate limiting
//! let global_limiter = global_rate_limit::<()>(1000, Duration::from_secs(60)); // 1000 requests per minute globally
//!
//! // Advanced configuration with builder pattern
//! let advanced_limiter = RateLimit::new(100, Duration::from_secs(60))
//!     .mode(RateLimitMode::PerIp)
//!     .with_burst_size(150)
//!     .with_error_message("Custom rate limit message")
//!     .with_retry_after(Duration::from_secs(300));
//! ```
//!
//! ### CORS
//!
//! ```ignore
//! use ruxno_middleware::{cors, CorsMiddleware};
//!
//! // Simple usage (development only - allows all origins)
//! let simple_cors = cors();
//!
//! // Production configuration
//! let production_cors = CorsMiddleware::new()
//!     .allow_origin("https://example.com")
//!     .allow_methods(&["GET", "POST", "PUT", "DELETE"])
//!     .allow_headers(&["Content-Type", "Authorization"])
//!     .allow_credentials(true)
//!     .max_age(3600);
//! ```
//!
//! ### Pretty JSON
//!
//! ```ignore
//! use ruxno_middleware::{pretty_json, PrettyJsonMiddleware};
//!
//! // Simple usage with default settings (2-space indentation)
//! let simple_pretty = pretty_json();
//!
//! // Or with custom configuration
//! let custom_pretty = PrettyJsonMiddleware::with_indent(4);
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

#[cfg(feature = "health-check")]
pub mod health_check;

// Re-exports for convenience
#[cfg(feature = "rate-limit")]
pub use rate_limit::{
    global_rate_limit, rate_limit, RateLimit, RateLimitMiddleware, RateLimitMode,
};

#[cfg(feature = "cors")]
pub use cors::{cors, CorsMiddleware};

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

#[cfg(feature = "health-check")]
pub use health_check::{
    health_check, health_check_with_config, simple_health_check, HealthCheckConfig,
    HealthCheckMiddleware, HealthCheckResult, HealthResponse, HealthStatus,
};
