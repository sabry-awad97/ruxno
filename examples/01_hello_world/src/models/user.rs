//! User Model
//!
//! Contains the User data structure and related types.

use serde::{Deserialize, Serialize};

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}
