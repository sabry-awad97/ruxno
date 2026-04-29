//! Ruxno - A fast, ergonomic Rust web framework with clean architecture
//!
//! # Architecture
//!
//! Ruxno follows a layered clean architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Application (App Facade)        │  ← Public API
//! ├─────────────────────────────────────────┤
//! │      Pipeline (Dispatcher/Executor)     │  ← Orchestration
//! ├─────────────────────────────────────────┤
//! │    Domain (Request/Response/Context)    │  ← Business Models
//! ├─────────────────────────────────────────┤
//! │      Core (Handler/Middleware Traits)   │  ← Abstractions
//! ├─────────────────────────────────────────┤
//! │   Infrastructure (Server/HTTP/Hyper)    │  ← External Dependencies
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use ruxno::{App, Context, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     app.get("/", async |c: Context| {
//!         Ok(c.text("Hello, World!"))
//!     });
//!
//!     app.listen("127.0.0.1:3000").await.unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core layer - Zero dependencies
pub mod core;

// Domain layer - Business models
pub mod domain;

// Routing layer - Pure logic
mod routing;

// Pipeline layer - Orchestration
mod pipeline;

// HTTP layer - Protocol abstractions
pub mod http;

// Body layer - Parsing
mod body;

// Upgrade layer - WebSocket/SSE
mod upgrade;

// Server layer - Infrastructure
pub mod server;

// App layer - Public facade
pub mod app;

// Prelude for convenience
pub mod prelude;

// Re-exports for convenience
pub use app::App;
pub use core::{Handler, Middleware};
pub use domain::{Context, Request, Response};
pub use server::{Server, ServerConfig};

/// Current version of the framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
