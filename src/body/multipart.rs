//! Multipart body parser

use crate::body::BodyParser;
use crate::core::CoreError;
use bytes::Bytes;

/// Multipart parser
pub struct MultipartParser;

#[async_trait::async_trait]
impl BodyParser for MultipartParser {
    type Output = Vec<Part>;

    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
        // TODO: Parse multipart data
        todo!("Implement MultipartParser::parse")
    }
}

/// Multipart part
pub struct Part {
    /// Part name
    pub name: String,
    /// Part data
    pub data: Bytes,
}
