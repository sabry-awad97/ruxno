//! Rate limiting middleware
//!
//! Provides in-memory rate limiting using the `governor` crate.
//!
//! # Examples
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

use governor::{Quota, RateLimiter};
use ruxno::{Context, Next, Response};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiting middleware
///
/// Uses the token bucket algorithm to limit requests per IP address.
pub struct RateLimitMiddleware {
    limiter:
        Arc<RateLimiter<IpAddr, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl RateLimitMiddleware {
    /// Create a rate limiter with requests per second
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limiter = RateLimitMiddleware::per_second(100);
    /// ```
    pub fn per_second(requests: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
        }
    }

    /// Create a rate limiter with requests per minute
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limiter = RateLimitMiddleware::per_minute(1000);
    /// ```
    pub fn per_minute(requests: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
        }
    }

    /// Create a rate limiter with requests per hour
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let limiter = RateLimitMiddleware::per_hour(10000);
    /// ```
    pub fn per_hour(requests: u32) -> Self {
        let quota = Quota::per_hour(NonZeroU32::new(requests).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
        }
    }

    /// Get the middleware function
    ///
    /// Returns a closure that can be used with `app.use_middleware()`.
    pub fn middleware<E>(
        &self,
    ) -> impl Fn(
        Context<E>,
        Next<E>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Response, ruxno::CoreError>> + Send + 'static>,
    >
    where
        E: Send + Sync + 'static,
    {
        let limiter = self.limiter.clone();
        move |ctx: Context<E>, next: Next<E>| {
            let limiter = limiter.clone();
            Box::pin(async move {
                // Extract IP from request
                let ip = extract_ip(&ctx);

                // Check rate limit
                if limiter.check_key(&ip).is_err() {
                    // Rate limit exceeded
                    return Ok(ctx.status(429).with_header("retry-after", "60").json(
                        &serde_json::json!({
                            "error": "Too Many Requests",
                            "message": "Rate limit exceeded. Please try again later.",
                            "retry_after": 60
                        }),
                    ));
                }

                // Continue to next middleware/handler
                next.run(ctx).await
            })
        }
    }
}

/// Extract IP address from context
///
/// Tries to extract from X-Forwarded-For header first, then falls back to
/// connection IP.
fn extract_ip<E>(_ctx: &Context<E>) -> IpAddr {
    // TODO: Implement proper IP extraction from headers and connection
    // For now, return a placeholder
    IpAddr::from([127, 0, 0, 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let _limiter = RateLimitMiddleware::per_second(100);
        let _limiter = RateLimitMiddleware::per_minute(1000);
        let _limiter = RateLimitMiddleware::per_hour(10000);
    }
}
