//! Admin Controller
//!
//! Handles admin dashboard and administrative functions.

use crate::config::AppEnv;
use ruxno::prelude::*;

/// Admin dashboard
pub async fn dashboard(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let user_count = env.db.get_user_count().await.unwrap_or(0);

    Ok(ctx.json(&serde_json::json!({
        "dashboard": "Admin Dashboard",
        "app_name": env.app_name,
        "stats": {
            "users": user_count,
            "posts": 1234,
            "comments": 5678
        }
    })))
}

/// Admin actions
pub async fn admin_action(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    Ok(ctx.json(&serde_json::json!({
        "message": "Admin action executed",
        "action": "update_settings",
        "success": true
    })))
}
