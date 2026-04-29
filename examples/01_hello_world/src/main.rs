//! Hello World MVC Example
//!
//! Demonstrates MVC architecture with Ruxno:
//! - Models: Data structures and database logic
//! - Views: Response formatting (JSON in this case)
//! - Controllers: Request handlers and business logic
//! - Routes: URL routing and middleware configuration
//! - Services: Business logic and external integrations
//! - Middleware: Cross-cutting concerns
//! - Config: Application configuration

mod config;
mod controllers;
mod database;
mod middleware;
mod models;
mod routes;
mod services;
mod util;
mod views;

use config::AppEnv;
use middleware::logging_middleware;
use routes::configure_routes;

use ruxno::prelude::*;
use ruxno_middleware::{cors, pretty_json};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create application environment with in-memory database
    let mut app = App::with_env(AppEnv::new());

    // Configure global middleware
    configure_middleware(&mut app);

    // Configure all routes (including health check)
    configure_routes(&mut app);

    // Print server info and start
    util::print_server_info();
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

/// Configure global middleware
fn configure_middleware(app: &mut App<AppEnv>) {
    // Global logging middleware - applies to ALL requests (including health checks)
    app.r#use(logging_middleware);

    // CORS middleware - allow cross-origin requests (development mode)
    app.r#use(cors());

    // Pretty JSON middleware - formats all JSON responses
    app.r#use(pretty_json());
}
