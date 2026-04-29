//! Graceful shutdown

use std::time::Duration;
use tokio::sync::broadcast;

/// Graceful shutdown coordinator
pub struct GracefulShutdown {
    tx: broadcast::Sender<()>,
    timeout: Duration,
}

impl GracefulShutdown {
    /// Create new shutdown coordinator
    pub fn new(timeout: Duration) -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { tx, timeout }
    }

    /// Trigger shutdown
    pub fn shutdown(&self) {
        let _ = self.tx.send(());
    }

    /// Subscribe to shutdown signal
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.tx.subscribe()
    }

    /// Get timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}
