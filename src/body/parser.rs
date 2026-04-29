//! Body parser trait

use crate::core::CoreError;
use bytes::Bytes;

/// Body parser trait
#[async_trait::async_trait]
pub trait BodyParser: Send + Sync {
    /// Output type
    type Output;

    /// Parse body bytes
    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError>;
}
