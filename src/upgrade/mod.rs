//! Protocol upgrade layer

pub mod sse;
pub mod websocket;

mod detector;

pub(crate) use detector::UpgradeDetector;

/// Protocol upgrade type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeType {
    /// WebSocket upgrade
    WebSocket,
    /// Server-Sent Events
    SSE,
}
