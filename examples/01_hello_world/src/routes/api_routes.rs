//! API Routes
//!
//! Defines all API-related routes and their handlers.

use crate::config::AppEnv;
use crate::controllers::home_controller;
use crate::middleware::api_middleware;
use ruxno::prelude::*;

/// Configure API routes
pub fn configure_api_routes(app: &App<AppEnv>) {
    // API middleware - applies to all /api/* routes
    app.on(Method::GET, "/api/*", api_middleware);

    // API status endpoint
    app.get("/api/status", home_controller::api_status);
}
