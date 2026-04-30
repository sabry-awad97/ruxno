//! Middleware execution phases
//!
//! Defines when middleware runs in the request lifecycle.

/// Middleware execution phase
///
/// Determines when middleware runs relative to routing:
/// - `PreRouting`: Runs before route matching (no access to route params)
/// - `PostRouting`: Runs after route matching (has access to route params)
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::pipeline::MiddlewarePhase;
///
/// // Pre-routing middleware (CORS preflight, health checks)
/// dispatcher.register_middleware(
///     MiddlewarePhase::PreRouting,
///     cors_middleware,
///     None,
/// );
///
/// // Post-routing middleware (auth, validation)
/// dispatcher.register_middleware(
///     MiddlewarePhase::PostRouting,
///     auth_middleware,
///     Some(opts),
/// );
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewarePhase {
    /// Run before routing
    ///
    /// Pre-routing middleware executes before route matching occurs.
    /// Use this for:
    /// - CORS preflight requests
    /// - Health checks that should bypass routing
    /// - Early request rejection (rate limiting, IP blocking)
    /// - Global request logging
    ///
    /// **Important**: Pre-routing middleware CANNOT access route parameters
    /// because routing hasn't happened yet.
    PreRouting,

    /// Run after routing (default)
    ///
    /// Post-routing middleware executes after route matching.
    /// Use this for:
    /// - Authentication and authorization
    /// - Request validation
    /// - Logging with route context
    /// - Response transformation
    ///
    /// **Benefit**: Post-routing middleware HAS access to route parameters
    /// extracted during routing.
    #[default]
    PostRouting,
}
