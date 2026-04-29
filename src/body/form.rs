//! Form body parser

use crate::body::BodyParser;
use crate::core::CoreError;
use bytes::Bytes;
use std::collections::HashMap;

/// Form parser
pub struct FormParser;

#[async_trait::async_trait]
impl BodyParser for FormParser {
    type Output = HashMap<String, String>;

    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
        // TODO: Parse form data
        todo!("Implement FormParser::parse")
    }
}
