//! Health Routes
//!
//! Defines health check and monitoring routes.

use crate::config::AppEnv;
use crate::controllers::health_controller;
use ruxno::prelude::*;

/// Configure health routes
pub fn configure_health_routes(app: &mut App<AppEnv>) {
    // Health check endpoint
    app.get("/health", health_controller::health_check);
}
