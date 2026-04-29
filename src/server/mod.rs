//! Server layer - HTTP transport
//!
//! This module provides the HTTP server implementation using Hyper.
//! It handles:
//!
//! - TCP connection management
//! - HTTP/1.1 protocol handling
//! - Request/response conversion
//! - Graceful shutdown
//!
//! # Architecture
//!
//! The server follows a clean layered architecture:
//! - `Server` - Main server struct with configuration
//! - `RuxnoService` - Bridges Hyper and domain layer
//! - `TcpListener` - TCP connection abstraction
//! - `ServerConfig` - Server configuration
//! - `GracefulShutdown` - Shutdown coordination
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::app::App;
//! use ruxno::server::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = App::new();
//!     app.get("/", |ctx| async move {
//!         Ok(ctx.text("Hello, World!"))
//!     });
//!
//!     let server = Server::new(app);
//!     server.listen("127.0.0.1:3000").await?;
//!     Ok(())
//! }
//! ```

mod builder;
mod config;
mod listener;
mod service;
mod shutdown;

pub use builder::ServerBuilder;
pub use config::{ServerConfig, TlsConfig};
pub use listener::TcpListener;
pub use service::RuxnoService;
pub use shutdown::GracefulShutdown;

use crate::app::App;
use crate::core::CoreError;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// HTTP Server
///
/// Wraps a Ruxno App and provides HTTP server functionality using Hyper.
/// Supports configuration through builder pattern and graceful shutdown.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use ruxno::app::App;
/// use ruxno::server::Server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut app = App::new();
///     app.get("/", |ctx| async move {
///         Ok(ctx.text("Hello!"))
///     });
///
///     Server::new(app).listen("127.0.0.1:3000").await?;
///     Ok(())
/// }
/// ```
///
/// ## With Configuration
///
/// ```rust,ignore
/// use ruxno::app::App;
/// use ruxno::server::{Server, ServerConfig};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut app = App::new();
///     app.get("/", |ctx| async move {
///         Ok(ctx.text("Hello!"))
///     });
///
///     let config = ServerConfig::new()
///         .with_max_body_size(10 * 1024 * 1024)
///         .with_request_timeout(Duration::from_secs(60));
///
///     Server::new(app)
///         .with_config(config)
///         .listen("127.0.0.1:3000")
///         .await?;
///     Ok(())
/// }
/// ```
pub struct Server<E = ()> {
    app: Arc<App<E>>,
    config: ServerConfig,
}

impl<E> Server<E>
where
    E: Send + Sync + 'static,
{
    /// Create new server from an App
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::app::App;
    /// use ruxno::server::Server;
    ///
    /// let app = App::new();
    /// let server = Server::new(app);
    /// ```
    pub fn new(app: App<E>) -> Self {
        Self {
            app: Arc::new(app),
            config: ServerConfig::default(),
        }
    }

    /// Configure server with custom settings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::server::{Server, ServerConfig};
    /// use std::time::Duration;
    ///
    /// let config = ServerConfig::new()
    ///     .with_max_body_size(5 * 1024 * 1024)
    ///     .with_request_timeout(Duration::from_secs(30));
    ///
    /// let server = Server::new(app).with_config(config);
    /// ```
    pub fn with_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Get a reference to the server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get a reference to the app
    pub fn app(&self) -> &Arc<App<E>> {
        &self.app
    }

    /// Start listening for connections
    ///
    /// This starts the HTTP server and listens for incoming connections.
    /// The server will run until Ctrl+C is pressed, then gracefully shutdown.
    ///
    /// # Architecture
    ///
    /// 1. Binds to TCP address
    /// 2. Accepts incoming connections
    /// 3. Spawns a task for each connection (full concurrency)
    /// 4. Converts between Hyper and Ruxno types via RuxnoService
    /// 5. Dispatches to App for routing/middleware/handling
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::app::App;
    /// use ruxno::server::Server;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut app = App::new();
    ///     app.get("/", |ctx| async move {
    ///         Ok(ctx.text("Hello!"))
    ///     });
    ///
    ///     Server::new(app).listen("127.0.0.1:3000").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn listen(self, addr: &str) -> Result<(), CoreError> {
        // Set up default Ctrl+C shutdown
        let shutdown = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            println!("\n👋 Shutting down gracefully...");
        };

        self.listen_with_shutdown(addr, shutdown).await
    }

    /// Start listening with a custom shutdown signal
    ///
    /// This allows you to provide your own shutdown signal instead of using Ctrl+C.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::app::App;
    /// use ruxno::server::Server;
    /// use tokio::sync::oneshot;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut app = App::new();
    ///     app.get("/", |ctx| async move {
    ///         Ok(ctx.text("Hello!"))
    ///     });
    ///
    ///     let (tx, rx) = oneshot::channel();
    ///
    ///     // Spawn server
    ///     tokio::spawn(async move {
    ///         Server::new(app)
    ///             .listen_with_shutdown("127.0.0.1:3000", async {
    ///                 rx.await.ok();
    ///             })
    ///             .await
    ///     });
    ///
    ///     // Later: trigger shutdown
    ///     tx.send(()).ok();
    ///     Ok(())
    /// }
    /// ```
    pub async fn listen_with_shutdown<F>(self, addr: &str, shutdown: F) -> Result<(), CoreError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // Use configured bind address if not overridden
        let bind_addr = if addr.is_empty() {
            self.config.bind_addr()
        } else {
            addr
        };

        // Bind to TCP address
        let listener = TcpListener::bind(bind_addr)
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to bind to {}: {}", bind_addr, e)))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| CoreError::Internal(format!("Failed to get local address: {}", e)))?;

        println!("🚀 Ruxno server listening on http://{}", local_addr);

        // Create service
        let service = RuxnoService::new(Arc::clone(&self.app));

        // Create connection limiter semaphore if max_connections is configured
        let connection_semaphore = self
            .config
            .max_connections()
            .map(|max| Arc::new(Semaphore::new(max)));

        if let Some(max) = self.config.max_connections() {
            println!("📊 Connection limit: {} concurrent connections", max);
        } else {
            println!("⚠️  No connection limit (unlimited connections)");
        }

        tokio::pin!(shutdown);

        // Main server loop
        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    let (stream, peer_addr) = result.map_err(|e| {
                        CoreError::Internal(format!("Failed to accept connection: {}", e))
                    })?;

                    // Try to acquire connection permit if limit is configured
                    let permit = if let Some(ref semaphore) = connection_semaphore {
                        match semaphore.clone().try_acquire_owned() {
                            Ok(permit) => Some(permit),
                            Err(_) => {
                                // Connection limit reached - reject with 503
                                eprintln!("🚫 Connection limit reached, rejecting connection from {}", peer_addr);

                                // Send 503 Service Unavailable response
                                let io = TokioIo::new(stream);
                                tokio::spawn(async move {
                                    let response = hyper::Response::builder()
                                        .status(hyper::StatusCode::SERVICE_UNAVAILABLE)
                                        .header("content-type", "application/json")
                                        .header("retry-after", "5")
                                        .body(http_body_util::Full::new(bytes::Bytes::from(
                                            r#"{"error":"Service Unavailable","message":"Server connection limit reached. Please try again later."}"#
                                        )))
                                        .unwrap();

                                    let service_fn = service_fn(move |_req: hyper::Request<Incoming>| {
                                        let response = response.clone();
                                        async move { Ok::<_, std::convert::Infallible>(response) }
                                    });

                                    let _ = http1::Builder::new()
                                        .serve_connection(io, service_fn)
                                        .await;
                                });
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    let io = TokioIo::new(stream);
                    let service = service.clone();
                    let max_body_size = self.config.max_body_size();
                    let max_headers = self.config.max_headers();
                    let request_timeout = self.config.request_timeout();

                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        // Hold permit for the duration of the connection
                        let _permit = permit;

                        let service_fn = service_fn(move |req: hyper::Request<Incoming>| {
                            let service = service.clone();
                            async move { service.handle(req, max_body_size, max_headers).await }
                        });

                        let connection = http1::Builder::new()
                            .serve_connection(io, service_fn);

                        // Wrap connection handling with timeout if configured
                        let result = if let Some(timeout) = request_timeout {
                            match tokio::time::timeout(timeout, connection).await {
                                Ok(result) => result,
                                Err(_) => {
                                    eprintln!("⏱️  Request timeout from {} after {:?}", peer_addr, timeout);
                                    return;
                                }
                            }
                        } else {
                            connection.await
                        };

                        if let Err(e) = result {
                            eprintln!("Connection error from {}: {}", peer_addr, e);
                        }

                        // Permit is automatically released when _permit is dropped
                    });
                }
                // Handle shutdown signal
                _ = &mut shutdown => {
                    println!("Server stopped");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn test_server_creation() {
        let app = App::new();
        let server = Server::new(app);
        assert!(Arc::strong_count(server.app()) >= 1);
    }

    #[test]
    fn test_server_with_config() {
        let app = App::new();
        let config = ServerConfig::new().with_port(8080);
        let server = Server::new(app).with_config(config);
        assert_eq!(server.config().bind_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn test_server_config_accessor() {
        let app = App::new();
        let server = Server::new(app);
        assert_eq!(server.config().bind_addr(), "127.0.0.1:3000");
    }

    #[test]
    fn test_server_app_accessor() {
        let app = App::new();
        let server = Server::new(app);
        assert!(Arc::strong_count(server.app()) >= 1);
    }

    #[test]
    fn test_server_default_config() {
        let app = App::new();
        let server = Server::new(app);
        let config = server.config();
        assert_eq!(config.bind_addr(), "127.0.0.1:3000");
        assert_eq!(config.max_body_size(), 1024 * 1024);
    }
}
