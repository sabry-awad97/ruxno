//! SSE keep-alive

use std::time::Duration;

/// Keep-alive configuration
pub struct KeepAlive {
    /// Interval between keep-alive messages
    interval: Duration,
}

impl KeepAlive {
    /// Create new keep-alive
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    /// Get interval
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Start keep-alive task
    pub async fn start(self) {
        // TODO: Spawn keep-alive task
        todo!("Implement KeepAlive::start")
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self::new(Duration::from_secs(15))
    }
}
