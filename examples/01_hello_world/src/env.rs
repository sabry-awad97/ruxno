//! Application environment and configuration
//!
//! This module defines the application environment that contains
//! shared resources like the database, configuration, and other
//! services that need to be accessible throughout the application.

use crate::db::InMemoryDb;

/// Application environment containing shared resources
///
/// The environment is passed to all handlers through the context,
/// allowing them to access shared resources like the database,
/// configuration, and other services.
///
/// # Examples
///
/// ```rust,ignore
/// // Create environment
/// let env = AppEnv::new();
///
/// // Create app with environment
/// let app = App::with_env(env);
///
/// // Access in handler
/// app.get("/users", |ctx: Context<AppEnv>| async move {
///     let env = ctx.env();
///     let users = env.db.get_all_users()?;
///     Ok(ctx.json(&users))
/// });
/// ```
#[derive(Debug, Clone)]
pub struct AppEnv {
    /// In-memory database instance
    pub db: InMemoryDb,

    /// Application name
    pub app_name: String,

    /// Application version
    pub version: String,

    /// Environment name (development, staging, production)
    pub environment: String,
}

impl AppEnv {
    /// Create a new application environment with default values
    ///
    /// Initializes the environment with:
    /// - A new in-memory database with sample data
    /// - Default application name and version
    /// - Development environment
    pub fn new() -> Self {
        Self {
            db: InMemoryDb::new(),
            app_name: "Ruxno Hello World".to_string(),
            version: "1.0.0".to_string(),
            environment: "development".to_string(),
        }
    }
}

impl Default for AppEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_env() {
        let env = AppEnv::new();
        assert_eq!(env.app_name, "Ruxno Hello World");
        assert_eq!(env.version, "1.0.0");
        assert_eq!(env.environment, "development");
    }
}
