//! Core error types

use thiserror::Error;

/// Core error type for the framework
#[derive(Debug, Error)]
pub enum CoreError {
    /// Route not found
    #[error("Route not found: {0}")]
    NotFound(String),

    /// Method not allowed
    #[error("Method not allowed")]
    MethodNotAllowed,

    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl CoreError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            CoreError::NotFound(_) => 404,
            CoreError::MethodNotAllowed => 405,
            CoreError::BadRequest(_) => 400,
            CoreError::Internal(_) => 500,
            CoreError::Custom(_) => 500,
        }
    }
}
