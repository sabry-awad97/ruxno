//! HTTP abstraction layer

// Re-export domain types
pub use crate::core::{Method, StatusCode};
pub use crate::domain::{Request, Response, ResponseBody};

// HTTP-specific modules
pub mod body;
pub mod convert;
pub mod headers;

pub use body::Body;
pub use convert::{from_hyper_request, to_hyper_response};
pub use headers::Headers;
