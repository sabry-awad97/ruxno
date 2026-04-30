//! Admin Routes
//!
//! Defines all admin-related routes and their handlers.

use crate::config::AppEnv;
use crate::controllers::admin_controller;
use crate::middleware::admin_middleware;
use ruxno::prelude::*;

/// Configure admin routes
pub fn configure_admin_routes(app: &App<AppEnv>) {
    // Admin routes with middleware
    app.route("/admin")
        .r#use(admin_middleware)
        .get(admin_controller::dashboard)
        .post(admin_controller::admin_action);
}
