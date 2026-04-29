//! Routes module
//!
//! Contains all route definitions organized by domain.

pub mod admin_routes;
pub mod api_routes;
pub mod user_routes;

use crate::config::AppEnv;
use crate::controllers::home_controller;
use ruxno::prelude::*;

pub use admin_routes::*;
pub use api_routes::*;
pub use user_routes::*;

/// Configure all application routes
pub fn configure_routes(app: &mut App<AppEnv>) {
    // Home route
    app.get("/", home_controller::index);

    // Configure domain-specific routes
    configure_user_routes(app);
    configure_admin_routes(app);
    configure_api_routes(app);
}
