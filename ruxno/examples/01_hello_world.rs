//! Hello World example

use ruxno::prelude::*;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Global middleware - applies to all routes (use "*" pattern)
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

    // Path-specific middleware - applies to /api/* routes
    app.r#use(async |ctx: Context, next: Next| {
        println!("🔐 API route - checking auth...");
        // TODO: Check authentication
        next.run(ctx).await
    });

    // Traditional syntax
    app.get("/", async |c: Context| Ok(c.text("Hello, World!")));

    // Route builder syntax - multiple methods on same path
    app.route("/users")
        .get(async |c: Context| Ok(c.text("Get all users")))
        .post(async |c: Context| Ok(c.text("Create user")));

    // Route with inline middleware using route builder
    app.route("/admin")
        .r#use(async |ctx: Context, next: Next| {
            println!("🔐 Admin route - checking admin auth...");
            // TODO: Check admin authentication
            next.run(ctx).await
        })
        .get(async |c: Context| Ok(c.text("Admin dashboard")))
        .post(async |c: Context| Ok(c.text("Admin action")));

    // Routes with path parameters - using ? operator for clean error handling
    app.route("/users/:id")
        .get(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.text(format!("Get user {}", id)))
        })
        .put(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.text(format!("Update user {}", id)))
        })
        .delete(async |c: Context| {
            let id = c.req.param("id")?;
            Ok(c.text(format!("Delete user {}", id)))
        });

    // API routes (will have both global and /api/* middleware)
    app.get("/api/status", async |c: Context| {
        Ok(c.json(&serde_json::json!({
            "status": "ok",
            "version": "1.0.0"
        })))
    });

    println!("🚀 Server starting on http://127.0.0.1:3000");
    println!();
    println!("Routes:");
    println!("   GET    /                  (global middleware)");
    println!("   GET    /users             (global middleware)");
    println!("   POST   /users             (global middleware)");
    println!("   GET    /admin             (global + admin middleware)");
    println!("   POST   /admin             (global + admin middleware)");
    println!("   GET    /users/:id         (global middleware)");
    println!("   PUT    /users/:id         (global middleware)");
    println!("   DELETE /users/:id         (global middleware)");
    println!("   GET    /api/status        (global + /api/* middleware)");
    println!();
    println!("Middleware patterns:");
    println!("   *        - Global (all routes)");
    println!("   /api/*   - API routes only");
    println!("   /admin   - Admin routes only");

    app.listen("127.0.0.1:3000").await.unwrap();
}
