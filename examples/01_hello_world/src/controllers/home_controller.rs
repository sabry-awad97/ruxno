//! Home Controller
//!
//! Handles home page and general application routes.

use crate::config::AppEnv;
use ruxno::prelude::*;

/// Home page handler
pub async fn index(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let user_count = env.db.get_user_count().await.unwrap_or(0);

    Ok(ctx.json(&serde_json::json!({
        "message": "Hello, World!",
        "app_name": env.app_name,
        "version": env.version,
        "user_count": user_count,
        "endpoints": [
            "/",
            "/health",
            "/users",
            "/users/:id",
            "/admin",
            "/api/status"
        ]
    })))
}

/// API status endpoint
pub async fn api_status(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let user_count = env.db.get_user_count().await.unwrap_or(0);

    Ok(ctx.json(&serde_json::json!({
        "status": "ok",
        "app_name": env.app_name,
        "version": env.version,
        "uptime": 12345,
        "environment": "development",
        "database": {
            "type": "in_memory",
            "user_count": user_count
        },
        "features": {
            "pretty_json": true,
            "rate_limiting": false,
            "cors": true,
            "health_check": true
        }
    })))
}
