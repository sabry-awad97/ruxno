//! TCP listener abstraction
//!
//! This module provides a thin wrapper around `tokio::net::TcpListener` with
//! additional error handling and convenience methods for the Ruxno server.
//!
//! # Examples
//!
//! ```no_run
//! use ruxno::server::TcpListener;
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     let listener = TcpListener::bind("127.0.0.1:3000").await?;
//!     println!("Listening on {}", listener.local_addr()?);
//!
//!     loop {
//!         let (stream, addr) = listener.accept().await?;
//!         println!("Accepted connection from {}", addr);
//!         // Handle connection...
//!     }
//! }
//! ```

use std::io;
use std::net::SocketAddr;

/// TCP listener wrapper
///
/// Wraps `tokio::net::TcpListener` to provide a clean abstraction for
/// accepting incoming TCP connections.
#[derive(Debug)]
pub struct TcpListener {
    inner: tokio::net::TcpListener,
}

impl TcpListener {
    /// Bind to the specified address
    ///
    /// Creates a new TCP listener bound to the specified address. The address
    /// can be any format accepted by `tokio::net::TcpListener::bind`, including:
    /// - IP address with port: `"127.0.0.1:3000"`
    /// - Hostname with port: `"localhost:3000"`
    /// - IPv6 address: `"[::1]:3000"`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The address is invalid or cannot be parsed
    /// - The port is already in use
    /// - Permission is denied (e.g., binding to port < 1024 without privileges)
    /// - The address is not available on this system
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn bind(addr: &str) -> io::Result<Self> {
        let inner = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("Failed to bind to address '{}': {}", addr, e),
            )
        })?;
        Ok(Self { inner })
    }

    /// Accept an incoming connection
    ///
    /// Waits for an incoming connection and returns the TCP stream and the
    /// remote address of the peer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The listener has been closed
    /// - An I/O error occurs while accepting the connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///     
    ///     let (stream, addr) = listener.accept().await?;
    ///     println!("Connection from: {}", addr);
    ///     Ok(())
    /// }
    /// ```
    pub async fn accept(&self) -> io::Result<(tokio::net::TcpStream, SocketAddr)> {
        self.inner
            .accept()
            .await
            .map_err(|e| io::Error::new(e.kind(), format!("Failed to accept connection: {}", e)))
    }

    /// Get the local address this listener is bound to
    ///
    /// # Errors
    ///
    /// Returns an error if the listener is not bound to an address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:0").await?;
    ///     let addr = listener.local_addr()?;
    ///     println!("Listening on: {}", addr);
    ///     Ok(())
    /// }
    /// ```
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    /// Get the Time To Live (TTL) value for this socket
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///     let ttl = listener.ttl()?;
    ///     println!("TTL: {}", ttl);
    ///     Ok(())
    /// }
    /// ```
    pub fn ttl(&self) -> io::Result<u32> {
        self.inner.ttl()
    }

    /// Set the Time To Live (TTL) value for this socket
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///     listener.set_ttl(64)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_ttl(ttl)
    }

    /// Convert into the underlying `tokio::net::TcpListener`
    ///
    /// This consumes the wrapper and returns the inner Tokio listener,
    /// useful for advanced use cases that need direct access to Tokio APIs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruxno::server::TcpListener;
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///     let tokio_listener = listener.into_inner();
    ///     // Use tokio_listener directly...
    ///     Ok(())
    /// }
    /// ```
    pub fn into_inner(self) -> tokio::net::TcpListener {
        self.inner
    }
}

impl From<tokio::net::TcpListener> for TcpListener {
    fn from(inner: tokio::net::TcpListener) -> Self {
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bind_success() {
        // Bind to port 0 to get any available port
        let listener = TcpListener::bind("127.0.0.1:0").await;
        assert!(listener.is_ok());
    }

    #[tokio::test]
    async fn test_bind_localhost() {
        let listener = TcpListener::bind("localhost:0").await;
        assert!(listener.is_ok());
    }

    #[tokio::test]
    async fn test_bind_ipv6() {
        // Try to bind to IPv6 loopback
        let listener = TcpListener::bind("[::1]:0").await;
        // This might fail on systems without IPv6, so we just check it doesn't panic
        let _ = listener;
    }

    #[tokio::test]
    async fn test_bind_invalid_address() {
        let listener = TcpListener::bind("invalid:address").await;
        assert!(listener.is_err());
    }

    #[tokio::test]
    async fn test_bind_invalid_port() {
        let listener = TcpListener::bind("127.0.0.1:99999").await;
        assert!(listener.is_err());
    }

    #[tokio::test]
    async fn test_local_addr() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr();
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn test_local_addr_preserves_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Port should be assigned by OS
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn test_accept_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a task to connect to the listener
        let connect_handle =
            tokio::spawn(async move { tokio::net::TcpStream::connect(addr).await });

        // Accept the connection
        let accept_result = listener.accept().await;
        assert!(accept_result.is_ok());

        let (stream, peer_addr) = accept_result.unwrap();
        assert!(peer_addr.port() > 0);

        // Ensure the connection was successful
        let connect_result = connect_handle.await.unwrap();
        assert!(connect_result.is_ok());

        // Clean up
        drop(stream);
    }

    #[tokio::test]
    async fn test_accept_multiple_connections() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Connect multiple times
        for _ in 0..3 {
            let connect_handle =
                tokio::spawn(async move { tokio::net::TcpStream::connect(addr).await });

            let (stream, _) = listener.accept().await.unwrap();
            connect_handle.await.unwrap().unwrap();
            drop(stream);
        }
    }

    #[tokio::test]
    async fn test_ttl() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ttl = listener.ttl();
        assert!(ttl.is_ok());
        assert!(ttl.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_set_ttl() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let result = listener.set_ttl(64);
        assert!(result.is_ok());

        let ttl = listener.ttl().unwrap();
        assert_eq!(ttl, 64);
    }

    #[tokio::test]
    async fn test_into_inner() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let tokio_listener = listener.into_inner();
        let inner_addr = tokio_listener.local_addr().unwrap();

        assert_eq!(addr, inner_addr);
    }

    #[tokio::test]
    async fn test_from_tokio_listener() {
        let tokio_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = tokio_listener.local_addr().unwrap();

        let listener = TcpListener::from(tokio_listener);
        let wrapper_addr = listener.local_addr().unwrap();

        assert_eq!(addr, wrapper_addr);
    }

    #[tokio::test]
    async fn test_debug_impl() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let debug_str = format!("{:?}", listener);
        assert!(debug_str.contains("TcpListener"));
    }

    #[tokio::test]
    async fn test_error_message_on_bind_failure() {
        // Try to bind to an invalid address
        let result = TcpListener::bind("999.999.999.999:3000").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("Failed to bind to address"));
        assert!(err_msg.contains("999.999.999.999:3000"));
    }

    #[tokio::test]
    async fn test_concurrent_accepts() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let listener = std::sync::Arc::new(listener);

        // Spawn multiple accept tasks
        let mut handles = vec![];
        for _ in 0..3 {
            let listener = listener.clone();
            let handle = tokio::spawn(async move { listener.accept().await });
            handles.push(handle);
        }

        // Connect multiple times
        for _ in 0..3 {
            tokio::net::TcpStream::connect(addr).await.unwrap();
        }

        // All accepts should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_bind_to_specific_port() {
        // Find an available port by binding to 0 first
        let temp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = temp_listener.local_addr().unwrap().port();
        drop(temp_listener);

        // Now bind to that specific port
        let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await;
        assert!(listener.is_ok());
    }

    #[tokio::test]
    async fn test_peer_address_from_accept() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let connect_handle =
            tokio::spawn(async move { tokio::net::TcpStream::connect(server_addr).await });

        let (_, peer_addr) = listener.accept().await.unwrap();

        // Peer address should be from localhost
        assert_eq!(peer_addr.ip().to_string(), "127.0.0.1");

        connect_handle.await.unwrap().unwrap();
    }
}
