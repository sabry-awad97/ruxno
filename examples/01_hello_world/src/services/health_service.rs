//! Health Service
//!
//! Provides health check functionality for various system components.

use crate::config::AppEnv;
use ruxno::prelude::*;
use ruxno_middleware::HealthCheckResult;

/// Database health check
pub async fn database_health_check(ctx: Context<AppEnv>) -> HealthCheckResult {
    // Access database from environment context
    let env = ctx.env();

    // Perform actual database health check using is_healthy method
    if env.db.is_healthy().await {
        let count = env.db.get_user_count().await.unwrap_or(0);
        HealthCheckResult::healthy_with_message(format!("Database OK - {} users", count))
    } else {
        HealthCheckResult::unhealthy("Database is not healthy - lock poisoned or inaccessible")
    }
}

/// Memory health check
pub async fn memory_health_check(ctx: Context<AppEnv>) -> HealthCheckResult {
    // Access environment from context
    let env = ctx.env();

    // Get real system memory information
    let mut system = sysinfo::System::new_all();
    system.refresh_memory();

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let available_memory = system.available_memory();
    let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

    // Convert bytes to MB for readability
    let total_mb = total_memory / 1024 / 1024;
    let used_mb = used_memory / 1024 / 1024;
    let available_mb = available_memory / 1024 / 1024;

    // Determine health status based on memory usage
    if memory_usage_percent > 90.0 {
        HealthCheckResult::unhealthy(format!(
            "Memory usage critical: {:.1}% ({} MB used / {} MB total) - App: {}",
            memory_usage_percent, used_mb, total_mb, env.app_name
        ))
    } else if memory_usage_percent > 80.0 {
        HealthCheckResult::degraded(format!(
            "Memory usage high: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
            memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
        ))
    } else {
        HealthCheckResult::healthy_with_message(format!(
            "Memory OK: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
            memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
        ))
    }
}

/// External API health check
pub async fn external_api_health_check(_ctx: Context<AppEnv>) -> HealthCheckResult {
    // Simulate external API check (sometimes degraded)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    if rand::random::<f32>() > 0.8 {
        HealthCheckResult::degraded("External API responding slowly")
    } else {
        HealthCheckResult::healthy_with_message("External API OK")
    }
}
