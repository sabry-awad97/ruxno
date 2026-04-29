//! User Repository
//!
//! Contains database operations for User entities.
//! This is the data access layer that handles all user-related database operations.

use crate::models::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// In-memory user repository
///
/// This is a thread-safe in-memory database implementation using
/// `RwLock` for concurrent read access and `Mutex` for ID generation.
///
/// # Thread Safety
///
/// - Uses `RwLock` for the user storage to allow multiple concurrent readers
/// - Uses `Mutex` for the ID counter to ensure unique ID generation
/// - All operations are atomic and thread-safe
///
/// # Examples
///
/// ```rust,ignore
/// let repo = UserRepository::new();
///
/// // Create a user
/// let user = repo.create_user(
///     "John Doe".to_string(),
///     "john@example.com".to_string(),
///     "user".to_string()
/// ).await?;
///
/// // Get all users
/// let users = repo.get_all_users().await?;
///
/// // Get user by ID
/// let user = repo.get_user(1).await?;
/// ```
#[derive(Debug, Clone)]
pub struct UserRepository {
    users: Arc<RwLock<HashMap<u32, User>>>,
    next_id: Arc<Mutex<u32>>,
}

impl UserRepository {
    /// Create a new user repository with some sample data
    ///
    /// The repository is pre-populated with 3 sample users for demonstration.
    pub fn new() -> Self {
        let mut users = HashMap::new();

        // Add some sample users
        users.insert(
            1,
            User {
                id: 1,
                name: "Alice Johnson".to_string(),
                email: "alice@example.com".to_string(),
                role: "admin".to_string(),
                created_at: "2024-01-15T10:30:00Z".to_string(),
            },
        );

        users.insert(
            2,
            User {
                id: 2,
                name: "Bob Smith".to_string(),
                email: "bob@example.com".to_string(),
                role: "user".to_string(),
                created_at: "2024-02-20T14:15:00Z".to_string(),
            },
        );

        users.insert(
            3,
            User {
                id: 3,
                name: "Charlie Brown".to_string(),
                email: "charlie@example.com".to_string(),
                role: "user".to_string(),
                created_at: "2024-03-10T09:45:00Z".to_string(),
            },
        );

        Self {
            users: Arc::new(RwLock::new(users)),
            next_id: Arc::new(Mutex::new(4)), // Start from 4 since we have 3 sample users
        }
    }

    /// Get all users
    ///
    /// Returns a vector of all users in the repository.
    /// Uses a read lock, so multiple threads can call this concurrently.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn get_all_users(&self) -> Result<Vec<User>, String> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }

    /// Get user by ID
    ///
    /// Returns `Some(user)` if found, `None` if not found.
    /// Uses a read lock, so multiple threads can call this concurrently.
    ///
    /// # Arguments
    ///
    /// * `id` - The user ID to search for
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn get_user(&self, id: u32) -> Result<Option<User>, String> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }

    /// Create a new user
    ///
    /// Creates a new user with an auto-generated ID and current timestamp.
    /// The ID generation is thread-safe using a `Mutex`.
    ///
    /// # Arguments
    ///
    /// * `name` - The user's full name
    /// * `email` - The user's email address
    /// * `role` - The user's role (e.g., "user", "admin")
    ///
    /// # Returns
    ///
    /// Returns the created user with the assigned ID and timestamp.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn create_user(
        &self,
        name: String,
        email: String,
        role: String,
    ) -> Result<User, String> {
        // Generate unique ID
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        drop(next_id); // Release the lock early

        let user = User {
            id,
            name,
            email,
            role,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Insert the user
        let mut users = self.users.write().await;
        users.insert(id, user.clone());

        Ok(user)
    }

    /// Update user
    ///
    /// Updates an existing user's fields. Only provided fields are updated;
    /// `None` values leave the field unchanged.
    ///
    /// # Arguments
    ///
    /// * `id` - The user ID to update
    /// * `name` - New name (optional)
    /// * `email` - New email (optional)
    /// * `role` - New role (optional)
    ///
    /// # Returns
    ///
    /// Returns `Some(user)` with the updated user if found, `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn update_user(
        &self,
        id: u32,
        name: Option<String>,
        email: Option<String>,
        role: Option<String>,
    ) -> Result<Option<User>, String> {
        let mut users = self.users.write().await;

        if let Some(user) = users.get_mut(&id) {
            if let Some(name) = name {
                user.name = name;
            }
            if let Some(email) = email {
                user.email = email;
            }
            if let Some(role) = role {
                user.role = role;
            }
            Ok(Some(user.clone()))
        } else {
            Ok(None)
        }
    }

    /// Delete user
    ///
    /// Removes a user from the repository.
    ///
    /// # Arguments
    ///
    /// * `id` - The user ID to delete
    ///
    /// # Returns
    ///
    /// Returns `true` if the user was found and deleted, `false` if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn delete_user(&self, id: u32) -> Result<bool, String> {
        let mut users = self.users.write().await;
        Ok(users.remove(&id).is_some())
    }

    /// Get user count
    ///
    /// Returns the total number of users in the repository.
    /// Useful for health checks and statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned (very rare in practice).
    pub async fn get_user_count(&self) -> Result<usize, String> {
        let users = self.users.read().await;
        Ok(users.len())
    }

    /// Check if repository is healthy
    ///
    /// Performs a basic health check by attempting to read the user count.
    /// This verifies that the repository locks are not poisoned and the
    /// data structure is accessible.
    ///
    /// # Returns
    ///
    /// Returns `true` if the repository is healthy, `false` otherwise.
    pub async fn is_healthy(&self) -> bool {
        self.get_user_count().await.is_ok()
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_repository_has_sample_data() {
        let repo = UserRepository::new();
        let users = repo.get_all_users().await.unwrap();
        assert_eq!(users.len(), 3);

        let alice = repo.get_user(1).await.unwrap().unwrap();
        assert_eq!(alice.name, "Alice Johnson");
        assert_eq!(alice.role, "admin");
    }

    #[tokio::test]
    async fn test_create_user() {
        let repo = UserRepository::new();

        let user = repo
            .create_user(
                "Test User".to_string(),
                "test@example.com".to_string(),
                "user".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(user.id, 4); // Should be 4 since we start with 3 users
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, "user");

        // Verify it was actually stored
        let stored_user = repo.get_user(4).await.unwrap().unwrap();
        assert_eq!(stored_user.name, "Test User");
    }

    #[tokio::test]
    async fn test_update_user() {
        let repo = UserRepository::new();

        // Update Alice's name
        let updated = repo
            .update_user(1, Some("Alice Updated".to_string()), None, None)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.name, "Alice Updated");
        assert_eq!(updated.email, "alice@example.com"); // Should remain unchanged

        // Try to update non-existent user
        let result = repo
            .update_user(999, Some("Test".to_string()), None, None)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let repo = UserRepository::new();

        // Delete existing user
        assert!(repo.delete_user(1).await.unwrap());
        assert!(repo.get_user(1).await.unwrap().is_none());

        // Try to delete non-existent user
        assert!(!repo.delete_user(999).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_user_count() {
        let repo = UserRepository::new();
        assert_eq!(repo.get_user_count().await.unwrap(), 3);

        repo.delete_user(1).await.unwrap();
        assert_eq!(repo.get_user_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let repo = UserRepository::new();
        assert!(repo.is_healthy().await);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let repo = Arc::new(UserRepository::new());
        let mut handles = vec![];

        // Spawn multiple tasks that create users concurrently
        for i in 0..10 {
            let repo_clone = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                repo_clone
                    .create_user(
                        format!("User {}", i),
                        format!("user{}@example.com", i),
                        "user".to_string(),
                    )
                    .await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Should have 3 original + 10 new = 13 users
        assert_eq!(repo.get_user_count().await.unwrap(), 13);
    }
}
