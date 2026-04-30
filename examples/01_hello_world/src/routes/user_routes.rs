//! User Routes
//!
//! Defines all user-related routes and their handlers.

use crate::config::AppEnv;
use crate::controllers::user_controller;
use ruxno::prelude::*;

/// Configure user routes
pub fn configure_user_routes(app: &App<AppEnv>) {
    // User CRUD routes
    app.route("/users")
        .get(user_controller::get_users)
        .post(user_controller::create_user);

    // User by ID routes
    app.route("/users/:id")
        .get(user_controller::get_user_by_id)
        .put(user_controller::update_user)
        .delete(user_controller::delete_user);
}
