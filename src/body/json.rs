//! JSON body parser

use crate::body::BodyParser;
use crate::core::CoreError;
use bytes::Bytes;
use serde::de::DeserializeOwned;

/// JSON parser
pub struct JsonParser;

#[async_trait::async_trait]
impl BodyParser for JsonParser {
    type Output = serde_json::Value;

    async fn parse(&self, body: &Bytes) -> Result<Self::Output, CoreError> {
        // TODO: Parse JSON
        todo!("Implement JsonParser::parse")
    }
}

impl JsonParser {
    /// Parse JSON into specific type
    pub async fn parse_as<T: DeserializeOwned>(body: &Bytes) -> Result<T, CoreError> {
        // TODO: Parse JSON into T
        todo!("Implement JsonParser::parse_as")
    }
}
