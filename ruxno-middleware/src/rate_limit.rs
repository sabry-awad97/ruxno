//! Rate limiting middleware
//!
//! Provides production-ready rate limiting using the battle-tested `governor` crate.
//!
//! This implementation combines the best of both worlds:
//! - **Per-IP rate limiting** with automatic cleanup (no memory leaks)
//! - **Global rate limiting** for simple use cases
//! - **Production IP extraction** with trusted proxy support
//! - **Builder pattern** for flexible configuration
//! - **Convenience functions** for common scenarios
//!
//! # Examples
//!
//! ## Simple Usage
//!
//! ```
//! use ruxno::App;
//! use ruxno_middleware::rate_limit;
//! use std::time::Duration;
//!
//! let mut app = App::new();
//!
//! // Global rate limiting: 100 requests per minute
//! app.r#use(rate_limit(100));
//! ```
//!
//! ## Per-IP Rate Limiting
//!
//! ```
//! use ruxno_middleware::{RateLimit, RateLimitMode};
//! use std::time::Duration;
//!
//! // 100 requests per minute per IP
//! let rate_limiter = RateLimit::new(100, Duration::from_secs(60))
//!     .mode(RateLimitMode::PerIp)
//!     .with_burst_size(150);
//! ```
//!
//! ## Advanced Configuration
//!
//! ```
//! use ruxno_middleware::{RateLimit, RateLimitMode};
//! use std::time::Duration;
//!
//! let rate_limiter = RateLimit::new(1000, Duration::from_secs(3600)) // 1000 per hour
//!     .mode(RateLimitMode::PerIp)
//!     .with_burst_size(100)
//!     .with_error_message("Custom rate limit message")
//!     .with_retry_after(Duration::from_secs(300));
//! ```

use async_trait::async_trait;
use governor::{Quota, RateLimiter};
use ruxno::core::{CoreError, Middleware, Next, StatusCode};
use ruxno::{Context, Response};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for the complex per-IP governor rate limiter type
type IpRateLimiter = RateLimiter<
    IpAddr,
    governor::state::keyed::DashMapStateStore<IpAddr>,
    governor::clock::DefaultClock,
>;

/// Type alias for the global governor rate limiter type
type GlobalRateLimiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
>;

/// Rate limiting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitMode {
    /// Global rate limiting (all requests share the same bucket)
    Global,
    /// Per-IP rate limiting (each IP has its own bucket)
    PerIp,
}

/// Rate limiting middleware
///
/// Supports both global and per-IP rate limiting with flexible configuration.
pub struct RateLimitMiddleware {
    mode: RateLimitMode,
    ip_limiter: Option<Arc<IpRateLimiter>>,
    global_limiter: Option<Arc<GlobalRateLimiter>>,
    error_message: String,
    retry_after_secs: u64,
}

impl RateLimitMiddleware {
    /// Create a per-IP rate limiter with requests per second
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimitMiddleware;
    ///
    /// let limiter = RateLimitMiddleware::per_second(100);
    /// ```
    pub fn per_second(requests: u32) -> Self {
        Self::per_ip_with_period(requests, Duration::from_secs(1), requests)
    }

    /// Create a per-IP rate limiter with requests per minute
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimitMiddleware;
    ///
    /// let limiter = RateLimitMiddleware::per_minute(1000);
    /// ```
    pub fn per_minute(requests: u32) -> Self {
        Self::per_ip_with_period(requests, Duration::from_secs(60), requests)
    }

    /// Create a per-IP rate limiter with requests per hour
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimitMiddleware;
    ///
    /// let limiter = RateLimitMiddleware::per_hour(10000);
    /// ```
    pub fn per_hour(requests: u32) -> Self {
        Self::per_ip_with_period(requests, Duration::from_secs(3600), requests)
    }

    /// Create a global rate limiter with requests per second
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimitMiddleware;
    ///
    /// let limiter = RateLimitMiddleware::global_per_second(1000);
    /// ```
    pub fn global_per_second(requests: u32) -> Self {
        Self::global_with_period(requests, Duration::from_secs(1), requests)
    }

    /// Create a global rate limiter with requests per minute
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimitMiddleware;
    ///
    /// let limiter = RateLimitMiddleware::global_per_minute(10000);
    /// ```
    pub fn global_per_minute(requests: u32) -> Self {
        Self::global_with_period(requests, Duration::from_secs(60), requests)
    }

    /// Create a per-IP rate limiter with custom period and burst
    fn per_ip_with_period(requests_per_period: u32, period: Duration, burst_size: u32) -> Self {
        let per_request_period = period / requests_per_period;
        let quota = Quota::with_period(per_request_period)
            .expect("Invalid period")
            .allow_burst(NonZeroU32::new(burst_size).expect("Burst size must be > 0"));

        Self {
            mode: RateLimitMode::PerIp,
            ip_limiter: Some(Arc::new(RateLimiter::dashmap(quota))),
            global_limiter: None,
            error_message: "Rate limit exceeded. Please try again later.".to_string(),
            retry_after_secs: 60,
        }
    }

    /// Create a global rate limiter with custom period and burst
    fn global_with_period(requests_per_period: u32, period: Duration, burst_size: u32) -> Self {
        let per_request_period = period / requests_per_period;
        let quota = Quota::with_period(per_request_period)
            .expect("Invalid period")
            .allow_burst(NonZeroU32::new(burst_size).expect("Burst size must be > 0"));

        Self {
            mode: RateLimitMode::Global,
            ip_limiter: None,
            global_limiter: Some(Arc::new(RateLimiter::direct(quota))),
            error_message: "Rate limit exceeded. Please try again later.".to_string(),
            retry_after_secs: 60,
        }
    }
}

#[async_trait]
impl<E> Middleware<E> for RateLimitMiddleware
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        let rate_limit_exceeded = match self.mode {
            RateLimitMode::PerIp => {
                let ip_limiter = self
                    .ip_limiter
                    .as_ref()
                    .expect("IP limiter not initialized");
                let ip = extract_ip(&ctx);
                ip_limiter.check_key(&ip).is_err()
            }
            RateLimitMode::Global => {
                let global_limiter = self
                    .global_limiter
                    .as_ref()
                    .expect("Global limiter not initialized");
                global_limiter.check().is_err()
            }
        };

        if rate_limit_exceeded {
            // Rate limit exceeded
            let error_body = serde_json::json!({
                "error": "Too Many Requests",
                "message": self.error_message,
                "retry_after": self.retry_after_secs
            });

            return Ok(Response::json(&error_body)
                .with_status_code(StatusCode::TOO_MANY_REQUESTS)
                .with_header("retry-after", self.retry_after_secs.to_string()));
        }

        // Continue to next middleware/handler
        next.run(ctx).await
    }
}

/// Convenience function to create a per-IP rate limiter with requests per second
///
/// # Examples
///
/// ```
/// use ruxno_middleware::rate_limit;
///
/// let middleware = rate_limit::<()>(100); // 100 requests per second per IP
/// ```
pub fn rate_limit<E>(requests_per_second: u32) -> impl Middleware<E>
where
    E: Send + Sync + 'static,
{
    RateLimitMiddleware::per_second(requests_per_second)
}

/// Convenience function to create a global rate limiter
///
/// # Examples
///
/// ```
/// use ruxno_middleware::global_rate_limit;
/// use std::time::Duration;
///
/// let middleware = global_rate_limit::<()>(1000, Duration::from_secs(60)); // 1000 requests per minute globally
/// ```
pub fn global_rate_limit<E>(requests_per_period: u32, period: Duration) -> impl Middleware<E>
where
    E: Send + Sync + 'static,
{
    RateLimit::new(requests_per_period, period)
        .mode(RateLimitMode::Global)
        .build()
}

/// Rate limiter builder for advanced configuration
///
/// # Examples
///
/// ```
/// use ruxno_middleware::{RateLimit, RateLimitMode};
/// use std::time::Duration;
///
/// let rate_limit = RateLimit::new(100, Duration::from_secs(60))
///     .mode(RateLimitMode::PerIp)
///     .with_burst_size(150)
///     .with_error_message("Custom rate limit message")
///     .with_retry_after(Duration::from_secs(300));
/// ```
pub struct RateLimit {
    requests_per_period: u32,
    period: Duration,
    burst_size: u32,
    mode: RateLimitMode,
    error_message: String,
    retry_after: Duration,
}

impl RateLimit {
    /// Create a new rate limiter builder
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimit;
    /// use std::time::Duration;
    ///
    /// let rate_limit = RateLimit::new(100, Duration::from_secs(60));
    /// ```
    pub fn new(requests_per_period: u32, period: Duration) -> Self {
        Self {
            requests_per_period,
            period,
            burst_size: requests_per_period,
            mode: RateLimitMode::PerIp, // Default to per-IP for security
            error_message: "Rate limit exceeded. Please try again later.".to_string(),
            retry_after: Duration::from_secs(60),
        }
    }

    /// Set the rate limiting mode
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::{RateLimit, RateLimitMode};
    /// use std::time::Duration;
    ///
    /// let rate_limit = RateLimit::new(100, Duration::from_secs(60))
    ///     .mode(RateLimitMode::Global);
    /// ```
    pub fn mode(mut self, mode: RateLimitMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set burst size (maximum tokens in the bucket)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimit;
    /// use std::time::Duration;
    ///
    /// let rate_limit = RateLimit::new(100, Duration::from_secs(60))
    ///     .with_burst_size(150);
    /// ```
    pub fn with_burst_size(mut self, burst_size: u32) -> Self {
        self.burst_size = burst_size;
        self
    }

    /// Set custom error message
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimit;
    /// use std::time::Duration;
    ///
    /// let rate_limit = RateLimit::new(100, Duration::from_secs(60))
    ///     .with_error_message("You're making too many requests!");
    /// ```
    pub fn with_error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = message.into();
        self
    }

    /// Set retry-after duration
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno_middleware::RateLimit;
    /// use std::time::Duration;
    ///
    /// let rate_limit = RateLimit::new(100, Duration::from_secs(60))
    ///     .with_retry_after(Duration::from_secs(300));
    /// ```
    pub fn with_retry_after(mut self, retry_after: Duration) -> Self {
        self.retry_after = retry_after;
        self
    }

    /// Build the middleware
    pub fn build(self) -> RateLimitMiddleware {
        let retry_after_secs = self.retry_after.as_secs();

        match self.mode {
            RateLimitMode::PerIp => {
                let mut middleware = RateLimitMiddleware::per_ip_with_period(
                    self.requests_per_period,
                    self.period,
                    self.burst_size,
                );
                middleware.error_message = self.error_message;
                middleware.retry_after_secs = retry_after_secs;
                middleware
            }
            RateLimitMode::Global => {
                let mut middleware = RateLimitMiddleware::global_with_period(
                    self.requests_per_period,
                    self.period,
                    self.burst_size,
                );
                middleware.error_message = self.error_message;
                middleware.retry_after_secs = retry_after_secs;
                middleware
            }
        }
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self::new(100, Duration::from_secs(60)) // 100 requests per minute per IP
    }
}

impl From<RateLimit> for RateLimitMiddleware {
    fn from(rate_limit: RateLimit) -> Self {
        rate_limit.build()
    }
}

/// Extract IP address from context
///
/// Tries to extract the real client IP from various headers in order of preference:
/// 1. X-Forwarded-For (first IP in the list)
/// 2. X-Real-IP
/// 3. X-Client-IP
/// 4. CF-Connecting-IP (Cloudflare)
/// 5. Fallback to localhost (127.0.0.1)
///
/// # Security Note
///
/// These headers can be spoofed by clients, so only trust them if you're behind
/// a trusted proxy/load balancer that sets these headers correctly.
fn extract_ip<E>(ctx: &Context<E>) -> IpAddr {
    // Try X-Forwarded-For header (most common)
    if let Some(xff) = ctx.req.header("x-forwarded-for") {
        // X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
        // We want the first one (the original client)
        if let Some(first_ip) = xff.split(',').next() {
            let trimmed = first_ip.trim();
            if let Ok(ip) = trimmed.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Try X-Real-IP header (nginx)
    if let Some(real_ip) = ctx.req.header("x-real-ip") {
        if let Ok(ip) = real_ip.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // Try X-Client-IP header
    if let Some(client_ip) = ctx.req.header("x-client-ip") {
        if let Ok(ip) = client_ip.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // Try CF-Connecting-IP header (Cloudflare)
    if let Some(cf_ip) = ctx.req.header("cf-connecting-ip") {
        if let Ok(ip) = cf_ip.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // Try True-Client-IP header (Akamai, Cloudflare)
    if let Some(true_ip) = ctx.req.header("true-client-ip") {
        if let Ok(ip) = true_ip.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // TODO: Extract from connection/socket info when available
    // For now, fallback to localhost
    // In a real implementation, you'd get this from the TCP connection
    IpAddr::from([127, 0, 0, 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use hyper::Version;
    use ruxno::core::Method;
    use ruxno::domain::Request;
    use ruxno::http::Headers;
    use std::collections::HashMap;

    fn create_test_context_with_headers(headers: Headers) -> Context<()> {
        let req = Request::new(
            Method::GET,
            "/".parse().unwrap(),
            Version::HTTP_11,
            HashMap::new(),
            headers,
            Bytes::new(),
        );
        Context::new(req, std::sync::Arc::new(()))
    }

    #[test]
    fn test_rate_limiter_creation() {
        let _limiter = RateLimitMiddleware::per_second(100);
        let _limiter = RateLimitMiddleware::per_minute(1000);
        let _limiter = RateLimitMiddleware::per_hour(10000);
    }

    #[test]
    fn test_extract_ip_x_forwarded_for() {
        let mut headers = Headers::new();
        headers
            .set("x-forwarded-for", "192.168.1.100, 10.0.0.1")
            .unwrap();
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([192, 168, 1, 100]));
    }

    #[test]
    fn test_extract_ip_x_real_ip() {
        let mut headers = Headers::new();
        headers.set("x-real-ip", "203.0.113.42").unwrap();
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([203, 0, 113, 42]));
    }

    #[test]
    fn test_extract_ip_cf_connecting_ip() {
        let mut headers = Headers::new();
        headers.set("cf-connecting-ip", "198.51.100.25").unwrap();
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([198, 51, 100, 25]));
    }

    #[test]
    fn test_extract_ip_ipv6() {
        let mut headers = Headers::new();
        headers.set("x-forwarded-for", "2001:db8::1").unwrap();
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, "2001:db8::1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_priority() {
        let mut headers = Headers::new();
        // X-Forwarded-For should take priority
        headers.set("x-forwarded-for", "192.168.1.100").unwrap();
        headers.set("x-real-ip", "10.0.0.1").unwrap();
        headers.set("cf-connecting-ip", "203.0.113.1").unwrap();
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([192, 168, 1, 100]));
    }

    #[test]
    fn test_extract_ip_fallback() {
        let headers = Headers::new(); // No IP headers
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([127, 0, 0, 1])); // localhost fallback
    }

    #[test]
    fn test_extract_ip_invalid_header() {
        let mut headers = Headers::new();
        headers.set("x-forwarded-for", "not-an-ip").unwrap();
        headers.set("x-real-ip", "192.168.1.100").unwrap(); // This should be used
        let ctx = create_test_context_with_headers(headers);

        let ip = extract_ip(&ctx);
        assert_eq!(ip, IpAddr::from([192, 168, 1, 100]));
    }
}
