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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    PostRouting,
}

impl Default for MiddlewarePhase {
    fn default() -> Self {
        // Post-routing is the default since it's the most common use case
        Self::PostRouting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_phase() {
        assert_eq!(MiddlewarePhase::default(), MiddlewarePhase::PostRouting);
    }

    #[test]
    fn test_phase_equality() {
        assert_eq!(MiddlewarePhase::PreRouting, MiddlewarePhase::PreRouting);
        assert_eq!(MiddlewarePhase::PostRouting, MiddlewarePhase::PostRouting);
        assert_ne!(MiddlewarePhase::PreRouting, MiddlewarePhase::PostRouting);
    }

    #[test]
    fn test_phase_clone() {
        let phase = MiddlewarePhase::PreRouting;
        let cloned = phase.clone();
        assert_eq!(phase, cloned);
    }

    #[test]
    fn test_phase_copy() {
        let phase = MiddlewarePhase::PostRouting;
        let copied = phase;
        assert_eq!(phase, copied);
    }
}
