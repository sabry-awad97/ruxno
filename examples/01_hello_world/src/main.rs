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
//! # Middleware System
//!
//! This example demonstrates Ruxno's unified middleware system with explicit phases:
//!
//! ## Pre-Routing Middleware
//! Runs BEFORE route matching. Use for:
//! - CORS preflight requests
//! - Health checks that should bypass routing
//! - Early request rejection (rate limiting, IP blocking)
//!
//! **Important**: Pre-routing middleware CANNOT access route parameters.
//!
//! ```rust,ignore
//! app.use_before_routing(cors_middleware);
//! app.use_before_routing_on("/health", health_check);
//! ```
//!
//! ## Post-Routing Middleware (Default)
//! Runs AFTER route matching. Use for:
//! - Authentication and authorization
//! - Request validation
//! - Logging with route context
//! - Response transformation
//!
//! **Benefit**: Post-routing middleware HAS access to route parameters.
//!
//! ```rust,ignore
//! app.r#use(logger);
//! app.use_on("/api/*", auth);
//! app.on(Method::POST, "/api/*", validation);
//! ```
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
    let app = App::with_env(AppEnv::new());

    // Configure global middleware
    configure_middleware(&app);

    // Configure all routes (including health check)
    configure_routes(&app);

    // Print server info and start
    util::print_server_info();
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

/// Configure middleware with explicit phases
fn configure_middleware(app: &App<AppEnv>) {
    // ========================================================================
    // Pre-Routing Middleware (runs BEFORE routing)
    // ========================================================================
    // Use for: CORS preflight, health checks, early rejection
    // Note: Cannot access route parameters

    // CORS middleware - handles preflight requests before routing
    app.use_before_routing(cors());

    // ========================================================================
    // Post-Routing Middleware (runs AFTER routing) - DEFAULT
    // ========================================================================
    // Use for: Auth, validation, logging with route context
    // Has access to route parameters

    // HTTP sniffer middleware - logs detailed request information
    app.with_http_sniffer();

    // Global logging middleware - applies to ALL requests (including health checks)
    app.r#use(logging_middleware);

    // Pretty JSON middleware - formats all JSON responses
    app.r#use(pretty_json());
}
