//! Core abstractions with zero dependencies
//!
//! This module defines the fundamental traits and types that all other
//! layers depend on. It has no external dependencies.

mod error;
mod handler;
mod middleware;
mod types;

pub use error::CoreError;
pub use handler::{BoxedHandler, Handler};
pub use middleware::{Middleware, Next};
pub use types::{Method, StatusCode};

// Internal utilities (not part of public API)
pub(crate) use handler::make_handler;
