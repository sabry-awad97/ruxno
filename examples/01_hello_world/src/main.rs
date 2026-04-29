//! Hello World example
//!
//! Demonstrates:
//! - Pretty JSON middleware
//! - JSON responses
//! - Global and path-specific middleware
//! - Route builder pattern
//! - Path parameters

mod util;

use ruxno::prelude::*;
use ruxno_middleware::{
    HealthCheckConfig, HealthCheckResult, cors, health_check_with_config, pretty_json,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Health check middleware - responds to /health with custom checks
    let health_config = HealthCheckConfig::new()
        .with_path("/health")
        .with_check("database", async || {
            // Simulate database check
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            HealthCheckResult::healthy_with_message("Database connection OK")
        })
        .with_check("cache", async || {
            // Simulate cache check
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            HealthCheckResult::healthy_with_message("Cache connection OK")
        })
        .with_check("external_api", async || {
            // Simulate external API check (sometimes degraded)
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            if rand::random::<f32>() > 0.8 {
                HealthCheckResult::degraded("External API responding slowly")
            } else {
                HealthCheckResult::healthy_with_message("External API OK")
            }
        });

    app.r#use(health_check_with_config(health_config));

    // CORS middleware - allow cross-origin requests (development mode)
    app.r#use(cors());

    // Pretty JSON middleware - formats all JSON responses
    app.r#use(pretty_json());

    // Global middleware - applies to all routes
    app.r#use(async |ctx: Context, next: Next| {
        println!(
            "🔍 Request: {} {}",
            ctx.req.method().as_str(),
            ctx.req.path()
        );
        let response = next.run(ctx).await?;
        println!("✅ Response: {}", response.status());
        Ok(response)
    });

    // Path-specific middleware - applies only to /api/* routes
    app.on(Method::GET, "/api/*", async |ctx: Context, next: Next| {
        println!("🔐 API route - checking auth...");
        // TODO: Check authentication
        next.run(ctx).await
    });

    // Home route - JSON response
    app.get("/", async |c: Context| {
        Ok(c.json(&serde_json::json!({
            "message": "Hello, World!",
            "version": "1.0.0",
            "endpoints": [
                "/",
                "/health",
                "/users",
                "/users/:id",
                "/admin",
                "/api/status"
            ]
        })))
    });

    // Route builder syntax - multiple methods on same path
    app.route("/users")
        .get(async |c: Context| {
            Ok(c.json(&serde_json::json!({
                "users": [
                    { "id": 1, "name": "Alice", "email": "alice@example.com" },
                    { "id": 2, "name": "Bob", "email": "bob@example.com" },
                    { "id": 3, "name": "Charlie", "email": "charlie@example.com" }
                ],
                "total": 3
            })))
        })
        .post(async |c: Context| {
            Ok(c.json(&serde_json::json!({
                "message": "User created",
                "id": 123,
                "name": "New User",
                "email": "newuser@example.com"
            })))
        });

    // Route with inline middleware using route builder
    app.route("/admin")
        .r#use(async |ctx: Context, next: Next| {
            println!("🔐 Admin route - checking admin auth...");
            // TODO: Check admin authentication
            next.run(ctx).await
        })
        .get(async |c: Context| {
            Ok(c.json(&serde_json::json!({
                "dashboard": "Admin Dashboard",
                "stats": {
                    "users": 150,
                    "posts": 1234,
                    "comments": 5678
                }
            })))
        })
        .post(async |c: Context| {
            Ok(c.json(&serde_json::json!({
                "message": "Admin action executed",
                "action": "update_settings",
                "success": true
            })))
        });

    // Routes with path parameters - using ? operator for clean error handling
    app.route("/users/:id")
        .get(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.json(&serde_json::json!({
                "id": id,
                "name": "John Doe",
                "email": "john@example.com",
                "role": "user",
                "created_at": "2025-01-01T00:00:00Z"
            })))
        })
        .put(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.json(&serde_json::json!({
                "message": "User updated",
                "id": id,
                "name": "John Doe Updated",
                "email": "john.updated@example.com"
            })))
        })
        .delete(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.json(&serde_json::json!({
                "message": "User deleted",
                "id": id,
                "success": true
            })))
        });

    // API routes (will have both global and /api/* middleware)
    app.get("/api/status", async |c: Context| {
        Ok(c.json(&serde_json::json!({
            "status": "ok",
            "version": "1.0.0",
            "uptime": 12345,
            "environment": "development",
            "features": {
                "pretty_json": true,
                "rate_limiting": false,
                "cors": true,
                "health_check": true
            }
        })))
    });

    util::print_server_info();

    app.listen("127.0.0.1:3000").await.unwrap();
}
