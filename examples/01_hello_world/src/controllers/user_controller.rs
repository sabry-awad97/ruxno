//! User Controller
//!
//! Handles all user-related HTTP requests and business logic.

use crate::config::AppEnv;
use ruxno::prelude::*;

/// Get all users
pub async fn get_users(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    match env.db.get_all_users().await {
        Ok(users) => Ok(ctx.json(&serde_json::json!({
            "users": users,
            "total": users.len()
        }))),
        Err(e) => Ok(ctx
            .json(&serde_json::json!({
                "error": "Failed to fetch users",
                "message": e
            }))
            .with_status(500)),
    }
}

/// Create a new user
pub async fn create_user(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();

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
        Ok(user) => Ok(ctx
            .json(&serde_json::json!({
                "message": "User created successfully",
                "user": user
            }))
            .with_status(201)),
        Err(e) => Ok(ctx
            .json(&serde_json::json!({
                "error": "Failed to create user",
                "message": e
            }))
            .with_status(500)),
    }
}

/// Get user by ID
pub async fn get_user_by_id(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let id_str = ctx.req.param("id")?;

    match id_str.parse::<u32>() {
        Ok(id) => match env.db.get_user(id).await {
            Ok(Some(user)) => Ok(ctx.json(&user)),
            Ok(None) => Ok(ctx
                .json(&serde_json::json!({
                    "error": "User not found",
                    "id": id
                }))
                .with_status(404)),
            Err(e) => Ok(ctx
                .json(&serde_json::json!({
                    "error": "Database error",
                    "message": e
                }))
                .with_status(500)),
        },
        Err(_) => Ok(ctx
            .json(&serde_json::json!({
                "error": "Invalid user ID",
                "id": id_str
            }))
            .with_status(400)),
    }
}

/// Update user by ID
pub async fn update_user(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let id_str = ctx.req.param("id")?;

    match id_str.parse::<u32>() {
        Ok(id) => {
            // In a real app, you'd parse the request body for update data
            // For demo purposes, we'll update with sample data
            match env
                .db
                .update_user(
                    id,
                    Some("Updated User".to_string()),
                    Some("updated@example.com".to_string()),
                    None,
                )
                .await
            {
                Ok(Some(user)) => Ok(ctx.json(&serde_json::json!({
                    "message": "User updated successfully",
                    "user": user
                }))),
                Ok(None) => Ok(ctx
                    .json(&serde_json::json!({
                        "error": "User not found",
                        "id": id
                    }))
                    .with_status(404)),
                Err(e) => Ok(ctx
                    .json(&serde_json::json!({
                        "error": "Failed to update user",
                        "message": e
                    }))
                    .with_status(500)),
            }
        }
        Err(_) => Ok(ctx
            .json(&serde_json::json!({
                "error": "Invalid user ID",
                "id": id_str
            }))
            .with_status(400)),
    }
}

/// Delete user by ID
pub async fn delete_user(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let id_str = ctx.req.param("id")?;

    match id_str.parse::<u32>() {
        Ok(id) => match env.db.delete_user(id).await {
            Ok(true) => Ok(ctx.json(&serde_json::json!({
                "message": "User deleted successfully",
                "id": id
            }))),
            Ok(false) => Ok(ctx
                .json(&serde_json::json!({
                    "error": "User not found",
                    "id": id
                }))
                .with_status(404)),
            Err(e) => Ok(ctx
                .json(&serde_json::json!({
                    "error": "Failed to delete user",
                    "message": e
                }))
                .with_status(500)),
        },
        Err(_) => Ok(ctx
            .json(&serde_json::json!({
                "error": "Invalid user ID",
                "id": id_str
            }))
            .with_status(400)),
    }
}
