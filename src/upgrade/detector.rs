//! Upgrade detection logic

use crate::domain::Request;
use crate::upgrade::UpgradeType;

/// Upgrade detector
pub struct UpgradeDetector;

impl UpgradeDetector {
    /// Detect upgrade type from request
    pub fn detect(req: &Request) -> Option<UpgradeType> {
        // TODO: Check headers for upgrade
        // TODO: Return upgrade type
        todo!("Implement UpgradeDetector::detect")
    }

    /// Check if request is WebSocket upgrade
    pub fn is_websocket(req: &Request) -> bool {
        // TODO: Check WebSocket headers
        todo!("Implement UpgradeDetector::is_websocket")
    }

    /// Check if request is SSE
    pub fn is_sse(req: &Request) -> bool {
        // TODO: Check SSE headers
        todo!("Implement UpgradeDetector::is_sse")
    }
}
