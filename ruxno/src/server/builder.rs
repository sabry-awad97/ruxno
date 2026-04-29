//! Server builder

use crate::app::App;
use crate::server::{Server, ServerConfig};

/// Server builder
pub struct ServerBuilder<E = ()> {
    app: App<E>,
    config: ServerConfig,
}

impl<E> ServerBuilder<E>
where
    E: Send + Sync + 'static,
{
    /// Create new builder
    pub fn new(app: App<E>) -> Self {
        Self {
            app,
            config: ServerConfig::default(),
        }
    }

    /// Set configuration
    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Build server
    pub fn build(self) -> Server<E> {
        Server::new(self.app).with_config(self.config)
    }
}
