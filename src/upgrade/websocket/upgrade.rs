//! WebSocket upgrade

use crate::core::CoreError;
use crate::domain::{Context, Response};
use crate::upgrade::websocket::WebSocket;

/// WebSocket upgrade
pub struct WebSocketUpgrade<E = ()> {
    ctx: Context<E>,
}

impl<E> WebSocketUpgrade<E> {
    /// Create new upgrade
    pub fn new(ctx: Context<E>) -> Self {
        Self { ctx }
    }

    /// Perform upgrade
    pub async fn upgrade<F, Fut>(self, handler: F) -> Result<Response, CoreError>
    where
        F: FnOnce(WebSocket) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        // TODO: Perform WebSocket handshake
        // TODO: Spawn handler task
        // TODO: Return upgrade response
        todo!("Implement WebSocketUpgrade::upgrade")
    }
}
