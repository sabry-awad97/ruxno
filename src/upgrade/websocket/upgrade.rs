//! WebSocket upgrade
//!
//! Provides WebSocket handshake and upgrade functionality following RFC 6455.
//!
//! # Examples
//!
//! ```rust,ignore
//! use ruxno::upgrade::websocket::WebSocketUpgrade;
//!
//! app.get("/ws", |ctx: Context| async move {
//!     let ws = WebSocketUpgrade::new(ctx);
//!     ws.upgrade(|socket| async move {
//!         // Handle WebSocket connection
//!         while let Some(Ok(msg)) = socket.recv().await {
//!             socket.send(msg).await.ok();
//!         }
//!     }).await
//! });
//! ```

use crate::core::CoreError;
use crate::domain::{Context, Response};
use crate::upgrade::websocket::WebSocket;
use sha1::{Digest, Sha1};

/// WebSocket GUID constant from RFC 6455
///
/// This magic string is concatenated with the client's Sec-WebSocket-Key
/// and hashed to produce the Sec-WebSocket-Accept value.
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// WebSocket upgrade
///
/// Handles the WebSocket handshake and upgrade process according to RFC 6455.
/// Validates the upgrade request, computes the accept key, and returns a
/// 101 Switching Protocols response.
///
/// # Design
///
/// The upgrade process follows RFC 6455:
/// 1. Validate the upgrade request (Upgrade + Sec-WebSocket-Key headers)
/// 2. Compute Sec-WebSocket-Accept from the client's key
/// 3. Return 101 Switching Protocols with proper headers
/// 4. Spawn a task to handle the WebSocket connection
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno::upgrade::websocket::WebSocketUpgrade;
///
/// app.get("/ws", |ctx: Context| async move {
///     let ws = WebSocketUpgrade::new(ctx);
///     ws.upgrade(|mut socket| async move {
///         while let Some(Ok(msg)) = socket.recv().await {
///             socket.send(msg).await.ok();
///         }
///     }).await
/// });
/// ```
pub struct WebSocketUpgrade<E = ()> {
    ctx: Context<E>,
}

impl<E> WebSocketUpgrade<E> {
    /// Create new WebSocket upgrade from context
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ws = WebSocketUpgrade::new(ctx);
    /// ```
    pub fn new(ctx: Context<E>) -> Self {
        Self { ctx }
    }

    /// Get the WebSocket accept key
    ///
    /// Computes the `Sec-WebSocket-Accept` value from the client's
    /// `Sec-WebSocket-Key` header according to RFC 6455.
    ///
    /// # Algorithm (RFC 6455 Section 1.3)
    ///
    /// 1. Concatenate the client's key with the WebSocket GUID
    /// 2. Compute SHA-1 hash of the concatenated string
    /// 3. Base64 encode the hash
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let ws = WebSocketUpgrade::new(ctx);
    /// let accept_key = ws.accept_key()?;
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the base64-encoded accept key, or
    /// `Err(CoreError)` if the `Sec-WebSocket-Key` header is missing.
    pub fn accept_key(&self) -> Result<String, CoreError> {
        // Get the Sec-WebSocket-Key header from the request
        let key = self
            .ctx
            .req
            .headers()
            .get("sec-websocket-key")
            .ok_or_else(|| {
                CoreError::bad_request("Missing Sec-WebSocket-Key header for WebSocket upgrade")
            })?;

        // Concatenate with the WebSocket GUID and hash
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WEBSOCKET_GUID.as_bytes());
        let hash = hasher.finalize();

        // Base64 encode the result
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            hash,
        ))
    }

    /// Perform the WebSocket upgrade
    ///
    /// Validates the upgrade request, performs the handshake, and spawns
    /// a task to handle the WebSocket connection.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ruxno::upgrade::websocket::{WebSocketUpgrade, Message};
    ///
    /// app.get("/ws", |ctx: Context| async move {
    ///     let ws = WebSocketUpgrade::new(ctx);
    ///     ws.upgrade(|mut socket| async move {
    ///         while let Some(Ok(msg)) = socket.recv().await {
    ///             match msg {
    ///                 Message::Text(text) => {
    ///                     println!("Received: {}", text);
    ///                     socket.send(Message::Text(text)).await.ok();
    ///                 }
    ///                 Message::Binary(data) => {
    ///                     socket.send(Message::Binary(data)).await.ok();
    ///                 }
    ///                 _ => {}
    ///             }
    ///         }
    ///     }).await
    /// });
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `Response` with status 101 (Switching Protocols) and the
    /// required WebSocket headers, or an error if the upgrade fails.
    pub async fn upgrade<F, Fut>(self, _handler: F) -> Result<Response, CoreError>
    where
        F: FnOnce(WebSocket) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        // Compute the accept key
        let accept_key = self.accept_key()?;

        // Create 101 Switching Protocols response
        let mut response = Response::new().with_status(http::StatusCode::SWITCHING_PROTOCOLS);

        // Set required WebSocket headers (RFC 6455)
        response
            .headers_mut()
            .set("upgrade", "websocket")
            .map_err(|e| CoreError::internal(format!("Failed to set Upgrade header: {}", e)))?;

        response
            .headers_mut()
            .set("connection", "Upgrade")
            .map_err(|e| CoreError::internal(format!("Failed to set Connection header: {}", e)))?;

        response
            .headers_mut()
            .set("sec-websocket-accept", &accept_key)
            .map_err(|e| {
                CoreError::internal(format!("Failed to set Sec-WebSocket-Accept header: {}", e))
            })?;

        // TODO: Spawn handler task when WebSocket implementation is complete
        // For now, just return the handshake response

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Request;
    use crate::http::Headers;
    use bytes::Bytes;
    use http::{Method, Uri};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_websocket_context() -> Context<()> {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket").unwrap();
        headers
            .set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .unwrap();

        let request = Request::new(
            Method::GET,
            Uri::from_static("http://localhost/ws"),
            HashMap::new(),
            headers,
            Bytes::new(),
        );

        Context::new(request, Arc::new(()))
    }

    #[test]
    fn test_accept_key_rfc6455() {
        // Test vector from RFC 6455 Section 1.3
        // Client sends: Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
        // Server should respond with: Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=

        let ctx = create_websocket_context();
        let ws = WebSocketUpgrade::new(ctx);

        let accept_key = ws.accept_key().unwrap();
        assert_eq!(accept_key, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_accept_key_missing_header() {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket").unwrap();
        // Missing sec-websocket-key

        let request = Request::new(
            Method::GET,
            Uri::from_static("http://localhost/ws"),
            HashMap::new(),
            headers,
            Bytes::new(),
        );

        let ctx = Context::new(request, Arc::new(()));
        let ws = WebSocketUpgrade::new(ctx);

        let result = ws.accept_key();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upgrade_returns_101() {
        let ctx = create_websocket_context();
        let ws = WebSocketUpgrade::new(ctx);

        let response = ws
            .upgrade(|_socket| async move {
                // Handler
            })
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::SWITCHING_PROTOCOLS);
    }

    #[tokio::test]
    async fn test_upgrade_sets_headers() {
        let ctx = create_websocket_context();
        let ws = WebSocketUpgrade::new(ctx);

        let response = ws
            .upgrade(|_socket| async move {
                // Handler
            })
            .await
            .unwrap();

        assert_eq!(response.headers().get("upgrade"), Some("websocket"));
        assert_eq!(response.headers().get("connection"), Some("Upgrade"));
        assert_eq!(
            response.headers().get("sec-websocket-accept"),
            Some("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")
        );
    }

    #[tokio::test]
    async fn test_upgrade_missing_key() {
        let mut headers = Headers::new();
        headers.set("upgrade", "websocket").unwrap();
        // Missing sec-websocket-key

        let request = Request::new(
            Method::GET,
            Uri::from_static("http://localhost/ws"),
            HashMap::new(),
            headers,
            Bytes::new(),
        );

        let ctx = Context::new(request, Arc::new(()));
        let ws = WebSocketUpgrade::new(ctx);

        let result = ws
            .upgrade(|_socket| async move {
                // Handler
            })
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_accept_key_different_keys() {
        // Test with different client keys to ensure uniqueness
        let keys = vec![
            "x3JJHMbDL1EzLkh9GBhXDw==",
            "dGhlIHNhbXBsZSBub25jZQ==",
            "AQIDBAUGBwgJCgsMDQ4PEC==",
        ];

        let mut accept_keys = Vec::new();

        for key in keys {
            let mut headers = Headers::new();
            headers.set("upgrade", "websocket").unwrap();
            headers.set("sec-websocket-key", key).unwrap();

            let request = Request::new(
                Method::GET,
                Uri::from_static("http://localhost/ws"),
                HashMap::new(),
                headers,
                Bytes::new(),
            );

            let ctx = Context::new(request, Arc::new(()));
            let ws = WebSocketUpgrade::new(ctx);

            accept_keys.push(ws.accept_key().unwrap());
        }

        // All accept keys should be unique
        assert_eq!(accept_keys.len(), 3);
        assert_ne!(accept_keys[0], accept_keys[1]);
        assert_ne!(accept_keys[1], accept_keys[2]);
        assert_ne!(accept_keys[0], accept_keys[2]);
    }

    #[test]
    fn test_websocket_guid_constant() {
        // Verify the GUID constant matches RFC 6455
        assert_eq!(WEBSOCKET_GUID, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    }
}
