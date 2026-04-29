//! Prelude - Common imports

pub use crate::app::App;
pub use crate::core::{CoreError as RuxnoError, Handler, Method, Middleware, Next, StatusCode};
pub use crate::domain::{Context, Request, Response};
pub use crate::server::{Server, ServerConfig};
