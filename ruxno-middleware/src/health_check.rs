//! Health Check middleware
//!
//! Provides health check endpoints for monitoring and load balancers.
//! Supports custom health checks for dependencies like databases, caches, etc.
//!
//! # Examples
//!
//! ## Simple Usage
//!
//! ```
//! use ruxno_middleware::health_check;
//!
//! let middleware = health_check(); // Responds to /health
//! ```
//!
//! ## Advanced Configuration
//!
//! ```
//! use ruxno_middleware::{HealthCheckConfig, HealthCheckResult, health_check_with_config};
//!
//! let config = HealthCheckConfig::new()
//!     .with_path("/healthz")
//!     .with_check("database", |ctx| async move {
//!         // Access environment from context
//!         let env = ctx.env();
//!         // Check database connection using env config
//!         HealthCheckResult::healthy()
//!     })
//!     .with_check("cache", |ctx| async move {
//!         // Access environment from context  
//!         let env = ctx.env();
//!         // Check cache connection using env config
//!         HealthCheckResult::healthy()
//!     });
//!
//! let middleware = health_check_with_config(config);
//! ```

use async_trait::async_trait;
use ruxno::core::{CoreError, Middleware, Next};
use ruxno::{Context, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Health status of the application or a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Application is healthy and functioning normally
    Healthy,
    /// Application is degraded but still operational
    Degraded,
    /// Application is unhealthy and not functioning properly
    Unhealthy,
}

impl HealthStatus {
    /// Convert health status to HTTP status code
    pub fn to_http_status(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200,  // Still operational
            HealthStatus::Unhealthy => 503, // Service Unavailable
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status of the health check
    pub status: HealthStatus,
    /// Optional description or error message
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
    /// Duration of the health check in milliseconds
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration_ms: Option<u64>,
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            duration_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a healthy result with a message
    pub fn healthy_with_message(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: Some(message.into()),
            duration_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a degraded result
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            duration_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            duration_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = Some(duration.as_millis() as u64);
        self
    }
}

/// Type alias for context-aware health check functions
pub type ContextHealthCheckFn<E> = Arc<
    dyn Fn(Context<E>) -> Pin<Box<dyn Future<Output = HealthCheckResult> + Send>> + Send + Sync,
>;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: HealthStatus,
    /// Timestamp of the check (ISO 8601 format)
    pub timestamp: String,
    /// Individual check results
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub checks: HashMap<String, HealthCheckResult>,
    /// Total duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration_ms: Option<u64>,
}

/// Health check middleware configuration
#[derive(Clone)]
pub struct HealthCheckConfig<E = ()> {
    /// Path to respond to (default: "/health")
    pub path: String,
    /// Whether to include detailed check results (default: true)
    pub detailed: bool,
    /// Context-aware health checks
    pub context_checks: HashMap<String, ContextHealthCheckFn<E>>,
}

impl<E> Default for HealthCheckConfig<E> {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            detailed: true,
            context_checks: HashMap::new(),
        }
    }
}

impl<E> HealthCheckConfig<E> {
    /// Create a new health check configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the health check path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Set whether to include detailed results
    pub fn with_detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }

    /// Add a context-aware health check that can access the request context and environment
    ///
    /// # Examples
    /// ```
    /// use ruxno_middleware::{HealthCheckConfig, HealthCheckResult};
    ///
    /// let config = HealthCheckConfig::new()
    ///     .with_check("database", |ctx| async move {
    ///         // Access environment from context
    ///         let env = ctx.env();
    ///         // Check database connection using env config
    ///         HealthCheckResult::healthy_with_message("Database connection OK")
    ///     });
    /// ```
    pub fn with_check<F, Fut>(mut self, name: impl Into<String>, check: F) -> Self
    where
        E: Send + Sync + 'static,
        F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HealthCheckResult> + Send + 'static,
    {
        let check_fn: ContextHealthCheckFn<E> = Arc::new(move |ctx| Box::pin(check(ctx)));
        self.context_checks.insert(name.into(), check_fn);
        self
    }
}

/// Health check middleware
#[derive(Clone)]
pub struct HealthCheckMiddleware<E = ()> {
    config: HealthCheckConfig<E>,
}

impl<E> HealthCheckMiddleware<E> {
    /// Create a new health check middleware with default configuration
    pub fn new() -> Self {
        Self {
            config: HealthCheckConfig::default(),
        }
    }

    /// Create a health check middleware with custom configuration
    pub fn with_config(config: HealthCheckConfig<E>) -> Self {
        Self { config }
    }

    /// Set the health check path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.config.path = path.into();
        self
    }

    /// Set whether to include detailed results
    pub fn with_detailed(mut self, detailed: bool) -> Self {
        self.config.detailed = detailed;
        self
    }

    /// Add a context-aware health check
    pub fn with_check<F, Fut>(mut self, name: impl Into<String>, check: F) -> Self
    where
        E: Send + Sync + 'static,
        F: Fn(Context<E>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HealthCheckResult> + Send + 'static,
    {
        let check_fn: ContextHealthCheckFn<E> = Arc::new(move |ctx| Box::pin(check(ctx)));
        self.config.context_checks.insert(name.into(), check_fn);
        self
    }

    /// Execute all health checks
    async fn execute_checks(&self, ctx: &Context<E>) -> HealthResponse {
        let start = Instant::now();
        let mut checks = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        // Execute context-aware checks
        for (name, check_fn) in &self.config.context_checks {
            let check_start = Instant::now();
            let mut result = check_fn(ctx.clone()).await;
            result.duration_ms = Some(check_start.elapsed().as_millis() as u64);

            // Update overall status
            match result.status {
                HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                HealthStatus::Degraded if overall_status != HealthStatus::Unhealthy => {
                    overall_status = HealthStatus::Degraded
                }
                _ => {}
            }

            checks.insert(name.clone(), result);
        }

        // Create ISO 8601 timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timestamp_str = format!(
            "{}Z",
            chrono::DateTime::from_timestamp(timestamp as i64, 0)
                .unwrap_or_default()
                .format("%Y-%m-%dT%H:%M:%S")
        );

        HealthResponse {
            status: overall_status,
            timestamp: timestamp_str,
            checks: if self.config.detailed {
                checks
            } else {
                HashMap::new()
            },
            duration_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

impl<E> Default for HealthCheckMiddleware<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> Middleware<E> for HealthCheckMiddleware<E>
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        // Check if this is a health check request
        if ctx.req.path() == self.config.path {
            let health_response = self.execute_checks(&ctx).await;
            let status_code = health_response.status.to_http_status();

            // Try to create a simple JSON response first
            let json_str = serde_json::to_string(&health_response).map_err(|e| {
                CoreError::Internal(format!("Failed to serialize health response: {}", e))
            })?;

            let status = ruxno::core::StatusCode::from_u16(status_code)
                .unwrap_or(ruxno::core::StatusCode::INTERNAL_SERVER_ERROR);

            return Ok(Response::new()
                .with_status_code(status)
                .with_header("content-type", "application/json")
                .with_body(json_str));
        }

        // Not a health check request, continue to next middleware
        next.run(ctx).await
    }
}

/// Create a health check middleware with default configuration
///
/// Responds to `/health` with a simple health status.
///
/// # Examples
/// ```
/// use ruxno_middleware::health_check;
///
/// let middleware = health_check();
/// ```
pub fn health_check<E>() -> impl Middleware<E>
where
    E: Send + Sync + 'static,
{
    HealthCheckMiddleware::new()
}

/// Create a health check middleware with custom configuration
///
/// # Examples
/// ```
/// use ruxno_middleware::{health_check_with_config, HealthCheckConfig, HealthCheckResult};
///
/// let config = HealthCheckConfig::new()
///     .with_path("/healthz")
///     .with_check("database", |ctx| async move {
///         // Access environment from context
///         let env = ctx.env();
///         // Check database connection using env config
///         HealthCheckResult::healthy()
///     })
///     .with_check("cache", |ctx| async move {
///         // Access environment from context
///         let env = ctx.env();
///         // Check cache connection
///         HealthCheckResult::healthy()
///     });
///
/// let middleware = health_check_with_config(config);
/// ```
pub fn health_check_with_config<E>(config: HealthCheckConfig<E>) -> impl Middleware<E>
where
    E: Send + Sync + 'static,
{
    HealthCheckMiddleware::with_config(config)
}

/// Create a simple health check that always returns healthy
///
/// Useful for basic liveness probes.
///
/// # Examples
/// ```
/// use ruxno_middleware::simple_health_check;
///
/// let middleware = simple_health_check("/healthz");
/// ```
pub fn simple_health_check<E>(path: &str) -> impl Middleware<E>
where
    E: Send + Sync + 'static,
{
    let path = path.to_string();

    struct SimpleHealthCheck {
        path: String,
    }

    #[async_trait]
    impl<E> Middleware<E> for SimpleHealthCheck
    where
        E: Send + Sync + 'static,
    {
        async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
            if ctx.req.path() == self.path {
                return Ok(Response::json(&serde_json::json!({
                    "status": "healthy"
                })));
            }
            next.run(ctx).await
        }
    }

    SimpleHealthCheck { path }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_health_status_http_codes() {
        assert_eq!(HealthStatus::Healthy.to_http_status(), 200);
        assert_eq!(HealthStatus::Degraded.to_http_status(), 200);
        assert_eq!(HealthStatus::Unhealthy.to_http_status(), 503);
    }

    #[test]
    fn test_health_check_result_creation() {
        let healthy = HealthCheckResult::healthy();
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.message.is_none());

        let degraded = HealthCheckResult::degraded("Slow response");
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.message, Some("Slow response".to_string()));

        let unhealthy = HealthCheckResult::unhealthy("Connection failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_health_check_result_with_metadata() {
        let result = HealthCheckResult::healthy()
            .with_metadata("version", serde_json::json!("1.0.0"))
            .with_duration(Duration::from_millis(50));

        assert_eq!(
            result.metadata.get("version"),
            Some(&serde_json::json!("1.0.0"))
        );
        assert_eq!(result.duration_ms, Some(50));
    }

    #[tokio::test]
    async fn test_health_check_middleware() {
        use ruxno::core::Method;
        use ruxno::domain::Request;
        use ruxno::http::Headers;
        use std::collections::HashMap;

        let middleware = HealthCheckMiddleware::new()
            .with_path("/test-health")
            .with_check("test", |_ctx| async move {
                sleep(Duration::from_millis(10)).await;
                HealthCheckResult::healthy_with_message("Test passed")
            });

        // Create a mock context
        let req = Request::new(
            Method::GET,
            "/test-health".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            bytes::Bytes::new(),
        );
        let ctx = ruxno::Context::new(req, std::sync::Arc::new(()));

        let response = middleware.execute_checks(&ctx).await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.checks.contains_key("test"));
        assert!(response.duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_health_check_with_failing_check() {
        use ruxno::core::Method;
        use ruxno::domain::Request;
        use ruxno::http::Headers;
        use std::collections::HashMap;

        let middleware = HealthCheckMiddleware::new()
            .with_check("failing", |_ctx| async move {
                HealthCheckResult::unhealthy("Test failure")
            })
            .with_check(
                "passing",
                |_ctx| async move { HealthCheckResult::healthy() },
            );

        // Create a mock context
        let req = Request::new(
            Method::GET,
            "/health".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            bytes::Bytes::new(),
        );
        let ctx = ruxno::Context::new(req, std::sync::Arc::new(()));

        let response = middleware.execute_checks(&ctx).await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
        assert_eq!(response.checks.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check_with_degraded_check() {
        use ruxno::core::Method;
        use ruxno::domain::Request;
        use ruxno::http::Headers;
        use std::collections::HashMap;

        let middleware = HealthCheckMiddleware::new()
            .with_check("degraded", |_ctx| async move {
                HealthCheckResult::degraded("Slow response")
            })
            .with_check(
                "healthy",
                |_ctx| async move { HealthCheckResult::healthy() },
            );

        // Create a mock context
        let req = Request::new(
            Method::GET,
            "/health".parse().unwrap(),
            HashMap::new(),
            Headers::new(),
            bytes::Bytes::new(),
        );
        let ctx = ruxno::Context::new(req, std::sync::Arc::new(()));

        let response = middleware.execute_checks(&ctx).await;
        assert_eq!(response.status, HealthStatus::Degraded);
    }
}
