//! Hyper conversion utilities

use crate::core::Method;
use crate::domain::{Request, Response};
use bytes::Bytes;
use hyper::body::Incoming;
use std::collections::HashMap;

/// Convert Hyper request to domain request
pub async fn from_hyper_request(req: hyper::Request<Incoming>) -> Request {
    // TODO: Extract method, path, headers, body
    // TODO: Buffer body
    // TODO: Create domain Request
    todo!("Implement from_hyper_request")
}

/// Convert domain response to Hyper response
pub fn to_hyper_response(res: Response) -> hyper::Response<http_body_util::Full<Bytes>> {
    // TODO: Convert status, headers, body
    // TODO: Create Hyper response
    todo!("Implement to_hyper_response")
}
