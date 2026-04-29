//! Server configuration

use std::time::Duration;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Maximum request body size
    pub max_body_size: usize,

    /// Request timeout
    pub request_timeout: Option<Duration>,

    /// Maximum number of headers
    pub max_headers: usize,

    /// Keep-alive timeout
    pub keep_alive_timeout: Option<Duration>,
}

impl ServerConfig {
    /// Create new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum body size
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Set maximum headers
    pub fn with_max_headers(mut self, max: usize) -> Self {
        self.max_headers = max;
        self
    }

    /// Set keep-alive timeout
    pub fn with_keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = Some(timeout);
        self
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_size: 1024 * 1024, // 1MB
            request_timeout: Some(Duration::from_secs(30)),
            max_headers: 100,
            keep_alive_timeout: Some(Duration::from_secs(60)),
        }
    }
}
