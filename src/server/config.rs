//! Server configuration
//!
//! This module provides configuration options for the HTTP server, including:
//! - Bind address and port
//! - Request timeouts and size limits
//! - Keep-alive settings
//! - TLS configuration (future support)
//!
//! # Examples
//!
//! ```
//! use ruxno::server::ServerConfig;
//! use std::time::Duration;
//!
//! let config = ServerConfig::new()
//!     .with_bind_addr("127.0.0.1:3000")
//!     .with_max_body_size(10 * 1024 * 1024) // 10MB
//!     .with_request_timeout(Duration::from_secs(60));
//! ```

use std::net::SocketAddr;
use std::time::Duration;

/// Server configuration
///
/// Provides builder pattern for configuring the HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (IP:port)
    bind_addr: String,

    /// Maximum request body size in bytes
    max_body_size: usize,

    /// Request timeout
    request_timeout: Option<Duration>,

    /// Maximum number of headers per request
    max_headers: usize,

    /// Keep-alive timeout
    keep_alive_timeout: Option<Duration>,

    /// Graceful shutdown timeout
    shutdown_timeout: Duration,

    /// Enable HTTP/1 support
    http1_enabled: bool,

    /// Enable HTTP/2 support (future)
    http2_enabled: bool,

    /// TLS configuration (future support)
    tls_config: Option<TlsConfig>,
}

/// TLS configuration (placeholder for future implementation)
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: String,

    /// Path to private key file
    pub key_path: String,
}

impl ServerConfig {
    /// Create new configuration with defaults
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new();
    /// assert_eq!(config.bind_addr(), "127.0.0.1:3000");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bind address
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .with_bind_addr("0.0.0.0:8080");
    ///
    /// assert_eq!(config.bind_addr(), "0.0.0.0:8080");
    /// ```
    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    /// Set port (keeps existing IP)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .with_port(8080);
    ///
    /// assert_eq!(config.bind_addr(), "127.0.0.1:8080");
    /// ```
    pub fn with_port(mut self, port: u16) -> Self {
        // Parse existing address and replace port
        if let Ok(addr) = self.bind_addr.parse::<SocketAddr>() {
            self.bind_addr = format!("{}:{}", addr.ip(), port);
        } else {
            // Fallback: assume it's just a port or invalid, use default IP
            self.bind_addr = format!("127.0.0.1:{}", port);
        }
        self
    }

    /// Set maximum body size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .with_max_body_size(10 * 1024 * 1024); // 10MB
    ///
    /// assert_eq!(config.max_body_size(), 10 * 1024 * 1024);
    /// ```
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Set request timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    /// use std::time::Duration;
    ///
    /// let config = ServerConfig::new()
    ///     .with_request_timeout(Duration::from_secs(60));
    ///
    /// assert_eq!(config.request_timeout(), Some(Duration::from_secs(60)));
    /// ```
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Disable request timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .without_request_timeout();
    ///
    /// assert_eq!(config.request_timeout(), None);
    /// ```
    pub fn without_request_timeout(mut self) -> Self {
        self.request_timeout = None;
        self
    }

    /// Set maximum number of headers
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .with_max_headers(200);
    ///
    /// assert_eq!(config.max_headers(), 200);
    /// ```
    pub fn with_max_headers(mut self, max: usize) -> Self {
        self.max_headers = max;
        self
    }

    /// Set keep-alive timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    /// use std::time::Duration;
    ///
    /// let config = ServerConfig::new()
    ///     .with_keep_alive_timeout(Duration::from_secs(120));
    ///
    /// assert_eq!(config.keep_alive_timeout(), Some(Duration::from_secs(120)));
    /// ```
    pub fn with_keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = Some(timeout);
        self
    }

    /// Disable keep-alive
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .without_keep_alive();
    ///
    /// assert_eq!(config.keep_alive_timeout(), None);
    /// ```
    pub fn without_keep_alive(mut self) -> Self {
        self.keep_alive_timeout = None;
        self
    }

    /// Set graceful shutdown timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    /// use std::time::Duration;
    ///
    /// let config = ServerConfig::new()
    ///     .with_shutdown_timeout(Duration::from_secs(60));
    ///
    /// assert_eq!(config.shutdown_timeout(), Duration::from_secs(60));
    /// ```
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Enable HTTP/2 support (future)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::ServerConfig;
    ///
    /// let config = ServerConfig::new()
    ///     .with_http2(true);
    ///
    /// assert_eq!(config.http2_enabled(), true);
    /// ```
    pub fn with_http2(mut self, enabled: bool) -> Self {
        self.http2_enabled = enabled;
        self
    }

    /// Set TLS configuration (future support)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruxno::server::{ServerConfig, TlsConfig};
    ///
    /// let tls = TlsConfig {
    ///     cert_path: "cert.pem".to_string(),
    ///     key_path: "key.pem".to_string(),
    /// };
    ///
    /// let config = ServerConfig::new()
    ///     .with_tls(tls);
    ///
    /// assert!(config.tls_config().is_some());
    /// ```
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls_config = Some(tls);
        self
    }

    // Accessor methods

    /// Get bind address
    pub fn bind_addr(&self) -> &str {
        &self.bind_addr
    }

    /// Get maximum body size
    pub fn max_body_size(&self) -> usize {
        self.max_body_size
    }

    /// Get request timeout
    pub fn request_timeout(&self) -> Option<Duration> {
        self.request_timeout
    }

    /// Get maximum headers
    pub fn max_headers(&self) -> usize {
        self.max_headers
    }

    /// Get keep-alive timeout
    pub fn keep_alive_timeout(&self) -> Option<Duration> {
        self.keep_alive_timeout
    }

    /// Get shutdown timeout
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }

    /// Check if HTTP/1 is enabled
    pub fn http1_enabled(&self) -> bool {
        self.http1_enabled
    }

    /// Check if HTTP/2 is enabled
    pub fn http2_enabled(&self) -> bool {
        self.http2_enabled
    }

    /// Get TLS configuration
    pub fn tls_config(&self) -> Option<&TlsConfig> {
        self.tls_config.as_ref()
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_config.is_some()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:3000".to_string(),
            max_body_size: 1024 * 1024, // 1MB
            request_timeout: Some(Duration::from_secs(30)),
            max_headers: 100,
            keep_alive_timeout: Some(Duration::from_secs(60)),
            shutdown_timeout: Duration::from_secs(30),
            http1_enabled: true,
            http2_enabled: false,
            tls_config: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_addr(), "127.0.0.1:3000");
        assert_eq!(config.max_body_size(), 1024 * 1024);
        assert_eq!(config.request_timeout(), Some(Duration::from_secs(30)));
        assert_eq!(config.max_headers(), 100);
        assert_eq!(config.keep_alive_timeout(), Some(Duration::from_secs(60)));
        assert_eq!(config.shutdown_timeout(), Duration::from_secs(30));
        assert!(config.http1_enabled());
        assert!(!config.http2_enabled());
        assert!(!config.is_tls_enabled());
    }

    #[test]
    fn test_new_config() {
        let config = ServerConfig::new();
        assert_eq!(config.bind_addr(), "127.0.0.1:3000");
    }

    #[test]
    fn test_with_bind_addr() {
        let config = ServerConfig::new().with_bind_addr("0.0.0.0:8080");
        assert_eq!(config.bind_addr(), "0.0.0.0:8080");
    }

    #[test]
    fn test_with_port() {
        let config = ServerConfig::new().with_port(8080);
        assert_eq!(config.bind_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn test_with_port_preserves_ip() {
        let config = ServerConfig::new()
            .with_bind_addr("0.0.0.0:3000")
            .with_port(9000);
        assert_eq!(config.bind_addr(), "0.0.0.0:9000");
    }

    #[test]
    fn test_with_max_body_size() {
        let config = ServerConfig::new().with_max_body_size(10 * 1024 * 1024);
        assert_eq!(config.max_body_size(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_with_request_timeout() {
        let config = ServerConfig::new().with_request_timeout(Duration::from_secs(60));
        assert_eq!(config.request_timeout(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_without_request_timeout() {
        let config = ServerConfig::new().without_request_timeout();
        assert_eq!(config.request_timeout(), None);
    }

    #[test]
    fn test_with_max_headers() {
        let config = ServerConfig::new().with_max_headers(200);
        assert_eq!(config.max_headers(), 200);
    }

    #[test]
    fn test_with_keep_alive_timeout() {
        let config = ServerConfig::new().with_keep_alive_timeout(Duration::from_secs(120));
        assert_eq!(config.keep_alive_timeout(), Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_without_keep_alive() {
        let config = ServerConfig::new().without_keep_alive();
        assert_eq!(config.keep_alive_timeout(), None);
    }

    #[test]
    fn test_with_shutdown_timeout() {
        let config = ServerConfig::new().with_shutdown_timeout(Duration::from_secs(60));
        assert_eq!(config.shutdown_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_with_http2() {
        let config = ServerConfig::new().with_http2(true);
        assert!(config.http2_enabled());
    }

    #[test]
    fn test_with_tls() {
        let tls = TlsConfig {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        };
        let config = ServerConfig::new().with_tls(tls);
        assert!(config.is_tls_enabled());
        assert!(config.tls_config().is_some());
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let config = ServerConfig::new()
            .with_bind_addr("0.0.0.0:8080")
            .with_max_body_size(5 * 1024 * 1024)
            .with_request_timeout(Duration::from_secs(45))
            .with_max_headers(150)
            .with_keep_alive_timeout(Duration::from_secs(90))
            .with_shutdown_timeout(Duration::from_secs(45))
            .with_http2(true);

        assert_eq!(config.bind_addr(), "0.0.0.0:8080");
        assert_eq!(config.max_body_size(), 5 * 1024 * 1024);
        assert_eq!(config.request_timeout(), Some(Duration::from_secs(45)));
        assert_eq!(config.max_headers(), 150);
        assert_eq!(config.keep_alive_timeout(), Some(Duration::from_secs(90)));
        assert_eq!(config.shutdown_timeout(), Duration::from_secs(45));
        assert!(config.http2_enabled());
    }

    #[test]
    fn test_clone() {
        let config1 = ServerConfig::new().with_port(8080);
        let config2 = config1.clone();
        assert_eq!(config1.bind_addr(), config2.bind_addr());
    }

    #[test]
    fn test_debug() {
        let config = ServerConfig::new();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ServerConfig"));
    }

    #[test]
    fn test_tls_config_debug() {
        let tls = TlsConfig {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        };
        let debug_str = format!("{:?}", tls);
        assert!(debug_str.contains("TlsConfig"));
    }

    #[test]
    fn test_tls_config_clone() {
        let tls1 = TlsConfig {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        };
        let tls2 = tls1.clone();
        assert_eq!(tls1.cert_path, tls2.cert_path);
        assert_eq!(tls1.key_path, tls2.key_path);
    }
}
