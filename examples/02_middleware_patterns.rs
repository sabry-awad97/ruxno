//! Middleware Patterns Example
//!
//! Demonstrates global and method + path-specific middleware.
//!
//! Run with:
//! ```bash
//! cargo run --example 02_middleware_patterns
//! ```
//!
//! Test with:
//! ```bash
//! # Global middleware applies to all routes
//! curl http://localhost:3000/
//! curl http://localhost:3000/public
//!
//! # All routes get global middleware
//! curl http://localhost:3000/api/users
//! curl http://localhost:3000/api/posts
//!
//! # Validation middleware applies only to POST /api/* routes
//! curl -X POST http://localhost:3000/api/users
//! curl -X GET http://localhost:3000/api/users  # No validation
//! ```

use ruxno::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Global middleware - applies to ALL routes
    app.r#use(|ctx: Context, next: Next| async move {
        println!("[Global] Request: {} {}", ctx.req.method(), ctx.req.path());
        let response = next.run(ctx).await?;
        println!("[Global] Response sent");
        Ok(response)
    });

    // Method + path-specific middleware - applies only to POST /api/* routes
    app.on(
        Method::POST,
        "/api/*",
        |ctx: Context, next: Next| async move {
            println!("[Validation] Validating POST request body");
            // In a real app, you'd validate the request body here
            let response = next.run(ctx).await?;
            println!("[Validation] Request validated");
            Ok(response)
        },
    );

    // Register routes
    app.get("/", |ctx: Context| async move {
        Ok(ctx.text("Home - only global middleware"))
    });

    app.get("/public", |ctx: Context| async move {
        Ok(ctx.text("Public - only global middleware"))
    });

    app.get("/api/users", |ctx: Context| async move {
        Ok(ctx.json(&serde_json::json!({
            "users": ["Alice", "Bob", "Charlie"]
        })))
    });

    app.post("/api/users", |ctx: Context| async move {
        Ok(ctx.json(&serde_json::json!({
            "message": "User created",
            "id": 123
        })))
    });

    app.get("/api/posts", |ctx: Context| async move {
        Ok(ctx.json(&serde_json::json!({
            "posts": ["Post 1", "Post 2", "Post 3"]
        })))
    });

    println!("🚀 Server running on http://localhost:3000");
    println!();
    println!("Try these commands:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/public");
    println!("  curl http://localhost:3000/api/users");
    println!("  curl -X POST http://localhost:3000/api/users");
    println!();

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
