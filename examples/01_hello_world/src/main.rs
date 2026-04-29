//! Hello World example
//!
//! Demonstrates:
//! - In-memory database with environment context
//! - Pretty JSON middleware
//! - JSON responses
//! - Global and path-specific middleware
//! - Route builder pattern
//! - Path parameters
//! - Context-aware health checks

mod db;
mod env;
mod util;

use env::AppEnv;
use ruxno::prelude::*;
use ruxno_middleware::{
    HealthCheckConfig, HealthCheckResult, cors, health_check_with_config, pretty_json,
};

#[tokio::main]
async fn main() {
    // Create application environment with in-memory database
    let env = AppEnv::new();
    let mut app = App::with_env(env);

    // Global logging middleware - applies to ALL requests (including health checks)
    app.r#use(async |ctx: Context<AppEnv>, next: Next<AppEnv>| {
        println!(
            "🔍 Request: {} {}",
            ctx.req.method().as_str(),
            ctx.req.path()
        );
        let response = next.run(ctx).await?;
        println!("✅ Response: {}", response.status());
        Ok(response)
    });

    // CORS middleware - allow cross-origin requests (development mode)
    app.r#use(cors());

    // Pretty JSON middleware - formats all JSON responses
    app.r#use(pretty_json());

    // Path-specific middleware - applies only to /api/* routes
    app.on(
        Method::GET,
        "/api/*",
        async |ctx: Context<AppEnv>, next: Next<AppEnv>| {
            println!("🔐 API route - checking auth...");
            // TODO: Check authentication
            next.run(ctx).await
        },
    );

    // Home route - JSON response with app info from environment
    app.get("/", async |c: Context<AppEnv>| {
        let env = c.env();
        let user_count = env.db.get_user_count().unwrap_or(0);

        Ok(c.json(&serde_json::json!({
            "message": "Hello, World!",
            "app_name": env.app_name,
            "version": env.version,
            "user_count": user_count,
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

    // Route builder syntax - multiple methods on same path with database access
    app.route("/users")
        .get(async |c: Context<AppEnv>| {
            let env = c.env();
            match env.db.get_all_users() {
                Ok(users) => Ok(c.json(&serde_json::json!({
                    "users": users,
                    "total": users.len()
                }))),
                Err(e) => Ok(c
                    .json(&serde_json::json!({
                        "error": "Failed to fetch users",
                        "message": e
                    }))
                    .with_status(500)),
            }
        })
        .post(async |c: Context<AppEnv>| {
            let env = c.env();

            // In a real app, you'd parse the request body here
            // For demo purposes, we'll create a user with sample data
            match env
                .db
                .create_user(
                    "New User".to_string(),
                    "newuser@example.com".to_string(),
                    "user".to_string(),
                )
                .await
            {
                Ok(user) => Ok(c
                    .json(&serde_json::json!({
                        "message": "User created successfully",
                        "user": user
                    }))
                    .with_status(201)),
                Err(e) => Ok(c
                    .json(&serde_json::json!({
                        "error": "Failed to create user",
                        "message": e
                    }))
                    .with_status(500)),
            }
        });

    // Route with inline middleware using route builder
    app.route("/admin")
        .r#use(async |ctx: Context<AppEnv>, next: Next<AppEnv>| {
            println!("🔐 Admin route - checking admin auth...");
            // TODO: Check admin authentication
            next.run(ctx).await
        })
        .get(async |c: Context<AppEnv>| {
            let env = c.env();
            let user_count = env.db.get_user_count().unwrap_or(0);

            Ok(c.json(&serde_json::json!({
                "dashboard": "Admin Dashboard",
                "app_name": env.app_name,
                "stats": {
                    "users": user_count,
                    "posts": 1234,
                    "comments": 5678
                }
            })))
        })
        .post(async |c: Context<AppEnv>| {
            Ok(c.json(&serde_json::json!({
                "message": "Admin action executed",
                "action": "update_settings",
                "success": true
            })))
        });

    // Routes with path parameters - using database to fetch real user data
    app.route("/users/:id")
        .get(async |c: Context<AppEnv>| {
            let env = c.env();
            let id_str = c.req.param("id")?;

            match id_str.parse::<u32>() {
                Ok(id) => match env.db.get_user(id) {
                    Ok(Some(user)) => Ok(c.json(&user)),
                    Ok(None) => Ok(c
                        .json(&serde_json::json!({
                            "error": "User not found",
                            "id": id
                        }))
                        .with_status(404)),
                    Err(e) => Ok(c
                        .json(&serde_json::json!({
                            "error": "Database error",
                            "message": e
                        }))
                        .with_status(500)),
                },
                Err(_) => Ok(c
                    .json(&serde_json::json!({
                        "error": "Invalid user ID",
                        "id": id_str
                    }))
                    .with_status(400)),
            }
        })
        .put(async |c: Context<AppEnv>| {
            let env = c.env();
            let id_str = c.req.param("id")?;

            match id_str.parse::<u32>() {
                Ok(id) => {
                    // In a real app, you'd parse the request body for update data
                    // For demo purposes, we'll update with sample data
                    match env.db.update_user(
                        id,
                        Some("Updated User".to_string()),
                        Some("updated@example.com".to_string()),
                        None,
                    ) {
                        Ok(Some(user)) => Ok(c.json(&serde_json::json!({
                            "message": "User updated successfully",
                            "user": user
                        }))),
                        Ok(None) => Ok(c
                            .json(&serde_json::json!({
                                "error": "User not found",
                                "id": id
                            }))
                            .with_status(404)),
                        Err(e) => Ok(c
                            .json(&serde_json::json!({
                                "error": "Failed to update user",
                                "message": e
                            }))
                            .with_status(500)),
                    }
                }
                Err(_) => Ok(c
                    .json(&serde_json::json!({
                        "error": "Invalid user ID",
                        "id": id_str
                    }))
                    .with_status(400)),
            }
        })
        .delete(async |c: Context<AppEnv>| {
            let env = c.env();
            let id_str = c.req.param("id")?;

            match id_str.parse::<u32>() {
                Ok(id) => match env.db.delete_user(id) {
                    Ok(true) => Ok(c.json(&serde_json::json!({
                        "message": "User deleted successfully",
                        "id": id
                    }))),
                    Ok(false) => Ok(c
                        .json(&serde_json::json!({
                            "error": "User not found",
                            "id": id
                        }))
                        .with_status(404)),
                    Err(e) => Ok(c
                        .json(&serde_json::json!({
                            "error": "Failed to delete user",
                            "message": e
                        }))
                        .with_status(500)),
                },
                Err(_) => Ok(c
                    .json(&serde_json::json!({
                        "error": "Invalid user ID",
                        "id": id_str
                    }))
                    .with_status(400)),
            }
        });

    // API routes (will have both global and /api/* middleware)
    app.get("/api/status", async |c: Context<AppEnv>| {
        let env = c.env();
        let user_count = env.db.get_user_count().unwrap_or(0);

        Ok(c.json(&serde_json::json!({
            "status": "ok",
            "app_name": env.app_name,
            "version": env.version,
            "uptime": 12345,
            "environment": "development",
            "database": {
                "type": "in_memory",
                "user_count": user_count
            },
            "features": {
                "pretty_json": true,
                "rate_limiting": false,
                "cors": true,
                "health_check": true
            }
        })))
    });

    // Health check middleware - responds to /health with database checks
    let health_config = HealthCheckConfig::new()
        .with_path("/health")
        .with_check("database", |ctx: Context<AppEnv>| async move {
            // Access database from environment context
            let env = ctx.env();

            // Perform actual database health check using is_healthy method
            if env.db.is_healthy() {
                let count = env.db.get_user_count().unwrap_or(0);
                HealthCheckResult::healthy_with_message(format!("Database OK - {} users", count))
            } else {
                HealthCheckResult::unhealthy(
                    "Database is not healthy - lock poisoned or inaccessible",
                )
            }
        })
        .with_check("memory", |ctx: Context<AppEnv>| async move {
            // Access environment from context
            let env = ctx.env();

            // Get real system memory information
            let mut system = sysinfo::System::new_all();
            system.refresh_memory();

            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let available_memory = system.available_memory();
            let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

            // Convert bytes to MB for readability
            let total_mb = total_memory / 1024 / 1024;
            let used_mb = used_memory / 1024 / 1024;
            let available_mb = available_memory / 1024 / 1024;

            // Determine health status based on memory usage
            if memory_usage_percent > 90.0 {
                HealthCheckResult::unhealthy(format!(
                    "Memory usage critical: {:.1}% ({} MB used / {} MB total) - App: {}",
                    memory_usage_percent, used_mb, total_mb, env.app_name
                ))
            } else if memory_usage_percent > 80.0 {
                HealthCheckResult::degraded(format!(
                    "Memory usage high: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
                    memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
                ))
            } else {
                HealthCheckResult::healthy_with_message(format!(
                    "Memory OK: {:.1}% ({} MB used / {} MB total, {} MB available) - App: {}",
                    memory_usage_percent, used_mb, total_mb, available_mb, env.app_name
                ))
            }
        })
        .with_check("external_api", |_ctx| async move {
            // Simulate external API check (sometimes degraded)
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            if rand::random::<f32>() > 0.8 {
                HealthCheckResult::degraded("External API responding slowly")
            } else {
                HealthCheckResult::healthy_with_message("External API OK")
            }
        });

    app.r#use(health_check_with_config(health_config));

    util::print_server_info();

    app.listen("127.0.0.1:3000").await.unwrap();
}
