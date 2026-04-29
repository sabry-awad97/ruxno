//! Health Controller
//!
//! Handles health check endpoints and system monitoring.

use crate::config::AppEnv;
use ruxno::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
    pub fn to_http_status(self) -> u16 {
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
}

impl HealthCheckResult {
    /// Create a healthy result with a message
    pub fn healthy_with_message(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: Some(message.into()),
            duration_ms: None,
        }
    }

    /// Create a degraded result
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            duration_ms: None,
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            duration_ms: None,
        }
    }

    /// Set the duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: HealthStatus,
    /// Timestamp of the check (ISO 8601 format)
    pub timestamp: String,
    /// Individual check results
    pub checks: HashMap<String, HealthCheckResult>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

/// Database health check
async fn database_health_check(ctx: &Context<AppEnv>) -> HealthCheckResult {
    let start = Instant::now();
    let env = ctx.env();

    // Perform actual database health check using is_healthy method
    if env.db.is_healthy().await {
        let count = env.db.get_user_count().await.unwrap_or(0);
        HealthCheckResult::healthy_with_message(format!("Database OK - {} users", count))
            .with_duration(start.elapsed().as_millis() as u64)
    } else {
        HealthCheckResult::unhealthy("Database is not healthy - lock poisoned or inaccessible")
            .with_duration(start.elapsed().as_millis() as u64)
    }
}

/// Memory health check
async fn memory_health_check(ctx: &Context<AppEnv>) -> HealthCheckResult {
    let start = Instant::now();
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

    let duration_ms = start.elapsed().as_millis() as u64;

    // Determine health status based on memory usage
    if memory_usage_percent > 90.0 {
        HealthCheckResult::unhealthy(format!(
            "Memory usage critical: {:.1}% ({} MB used / {} MB total) - App: {}",
            memory_usage_percent, used_mb, total_mb, env.app_name
        ))
        .with_duration(duration_ms)
    } else if memory_usage_percent > 80.0 {
        HealthCheckResult::degraded(format!(
            "Memory usage high: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
            memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
        ))
        .with_duration(duration_ms)
    } else {
        HealthCheckResult::healthy_with_message(format!(
            "Memory OK: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
            memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
        ))
        .with_duration(duration_ms)
    }
}

/// External API health check
async fn external_api_health_check(_ctx: &Context<AppEnv>) -> HealthCheckResult {
    let start = Instant::now();

    // Simulate external API check (sometimes degraded)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let duration_ms = start.elapsed().as_millis() as u64;

    if rand::random::<f32>() > 0.8 {
        HealthCheckResult::degraded("External API responding slowly").with_duration(duration_ms)
    } else {
        HealthCheckResult::healthy_with_message("External API OK").with_duration(duration_ms)
    }
}

/// Health check endpoint handler
pub async fn health_check(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let start = Instant::now();
    let mut checks = HashMap::new();
    let mut overall_status = HealthStatus::Healthy;

    // Execute all health checks
    let database_result = database_health_check(&ctx).await;
    let memory_result = memory_health_check(&ctx).await;
    let external_api_result = external_api_health_check(&ctx).await;

    // Update overall status based on individual check results
    for (name, result) in [
        ("database", &database_result),
        ("memory", &memory_result),
        ("external_api", &external_api_result),
    ] {
        match result.status {
            HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
            HealthStatus::Degraded if overall_status != HealthStatus::Unhealthy => {
                overall_status = HealthStatus::Degraded
            }
            _ => {}
        }
        checks.insert(name.to_string(), result.clone());
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

    let health_response = HealthResponse {
        status: overall_status,
        timestamp: timestamp_str,
        checks,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    let status_code = health_response.status.to_http_status();

    Ok(ctx.json(&health_response).with_status(status_code))
}
