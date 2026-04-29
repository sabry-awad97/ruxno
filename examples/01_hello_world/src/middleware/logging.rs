//! Logging Middleware
//!
//! Provides request/response logging functionality.

use crate::config::AppEnv;
use ruxno::prelude::*;

/// Global logging middleware - applies to ALL requests
pub async fn logging_middleware(
    ctx: Context<AppEnv>,
    next: Next<AppEnv>,
) -> Result<Response, RuxnoError> {
    println!(
        "🔍 Request: {} {}",
        ctx.req.method().as_str(),
        ctx.req.path()
    );
    let response = next.run(ctx).await?;
    println!("✅ Response: {}", response.status());
    Ok(response)
}

/// API-specific middleware - applies only to /api/* routes
pub async fn api_middleware(
    ctx: Context<AppEnv>,
    next: Next<AppEnv>,
) -> Result<Response, RuxnoError> {
    println!("🔐 API route - checking auth...");
    // TODO: Check authentication
    next.run(ctx).await
}

/// Admin-specific middleware - applies only to admin routes
pub async fn admin_middleware(
    ctx: Context<AppEnv>,
    next: Next<AppEnv>,
) -> Result<Response, RuxnoError> {
    println!("🔐 Admin route - checking admin auth...");
    // TODO: Check admin authentication
    next.run(ctx).await
}
