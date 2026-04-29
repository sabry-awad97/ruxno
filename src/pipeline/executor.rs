//! Executor - Handler execution logic

use crate::core::{BoxedHandler, CoreError};
use crate::domain::{Context, Response};

/// Executor - Executes handlers
pub struct Executor;

impl Executor {
    /// Execute a handler
    pub async fn execute<E>(
        handler: BoxedHandler<E>,
        ctx: Context<E>,
    ) -> Result<Response, CoreError>
    where
        E: Send + Sync + 'static,
    {
        // TODO: Execute handler
        // TODO: Handle errors
        // TODO: Convert to response
        todo!("Implement Executor::execute")
    }
}
