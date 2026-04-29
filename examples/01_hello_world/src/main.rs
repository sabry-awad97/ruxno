//! Hello World MVC Example
//!
//! Demonstrates MVC architecture with Ruxno:
//! - Models: Data structures and database logic
//! - Views: Response formatting (JSON in this case)
//! - Controllers: Request handlers and business logic
//! - Routes: URL routing and middleware configuration
//! - Services: Business logic and external integrations
//! - Middleware: Cross-cutting concerns (logging, HTTP sniffer, CORS, etc.)
//! - Config: Application configuration
//!
//! # HTTP Sniffer
//!
//! This example includes an HTTP sniffer middleware that logs detailed request
//! information. It captures:
//! - Timestamp (ISO 8601 format)
//! - HTTP method and version
//! - Request URL and parsed components
//! - All headers with enumerated output
//! - Query parameters
//!
//! # Usage
//!
//! ```bash
//! cargo run
//! ```
//!
//! Then make requests to see the detailed logging:
//!
//! ```bash
//! # Basic request
//! curl http://localhost:3000/
//!
//! # Request with query parameters
//! curl "http://localhost:3000/api/users?page=1&limit=10"
//!
//! # POST with JSON
//! curl -X POST http://localhost:3000/api/users \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"John","email":"john@example.com"}'
//! ```

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
use middleware::http_sniffer::HttpSnifferExt;
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
    // HTTP sniffer middleware - logs detailed request information
    app.with_http_sniffer();

    // Global logging middleware - applies to ALL requests (including health checks)
    app.r#use(logging_middleware);

    // CORS middleware - allow cross-origin requests (development mode)
    app.r#use(cors());

    // Pretty JSON middleware - formats all JSON responses
    app.r#use(pretty_json());
}
