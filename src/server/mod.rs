//! Server layer - HTTP transport

mod builder;
mod config;
mod listener;
mod service;
mod shutdown;

pub use builder::ServerBuilder;
pub use config::ServerConfig;
pub use listener::TcpListener;
pub use service::RuxnoService;
pub use shutdown::GracefulShutdown;

use crate::app::App;
use crate::core::CoreError;
use std::sync::Arc;

/// HTTP Server
pub struct Server<E = ()> {
    app: Arc<App<E>>,
    config: ServerConfig,
}

impl<E> Server<E>
where
    E: Send + Sync + 'static,
{
    /// Create new server
    pub fn new(app: App<E>) -> Self {
        Self {
            app: Arc::new(app),
            config: ServerConfig::default(),
        }
    }

    /// Configure server
    pub fn with_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Start listening
    pub async fn listen(self, addr: &str) -> Result<(), CoreError> {
        // TODO: Bind to address
        // TODO: Accept connections
        // TODO: Spawn connection handlers
        todo!("Implement Server::listen")
    }

    /// Start with graceful shutdown
    pub async fn listen_with_shutdown<F>(self, addr: &str, shutdown: F) -> Result<(), CoreError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // TODO: Bind to address
        // TODO: Accept connections with shutdown signal
        todo!("Implement Server::listen_with_shutdown")
    }
}
