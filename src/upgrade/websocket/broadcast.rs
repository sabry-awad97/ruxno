//! WebSocket broadcasting
//!
//! This module provides a broadcast channel for sending WebSocket messages to multiple
//! subscribers simultaneously. It's useful for implementing chat rooms, live updates,
//! and other scenarios where one message needs to reach many clients.
//!
//! # Example
//!
//! ```rust
//! use ruxno_clean::upgrade::websocket::{Broadcaster, Message};
//!
//! # async fn example() {
//! // Create broadcaster with capacity for 100 messages
//! let broadcaster = Broadcaster::new(100);
//!
//! // Subscribe multiple clients
//! let mut client1 = broadcaster.subscribe();
//! let mut client2 = broadcaster.subscribe();
//!
//! // Broadcast a message to all subscribers
//! broadcaster.broadcast(Message::text("Hello, everyone!")).ok();
//!
//! // Each client receives the message
//! assert_eq!(client1.recv().await.unwrap(), Message::text("Hello, everyone!"));
//! assert_eq!(client2.recv().await.unwrap(), Message::text("Hello, everyone!"));
//! # }
//! ```

use crate::upgrade::websocket::Message;
use tokio::sync::broadcast;

/// Broadcaster for WebSocket messages to multiple subscribers
///
/// Uses Tokio's broadcast channel internally, which provides:
/// - Multiple producers, multiple consumers (MPMC)
/// - Message cloning for each subscriber
/// - Automatic lagging subscriber handling
/// - Bounded capacity with overflow behavior
///
/// # Capacity and Overflow
///
/// The broadcaster has a fixed capacity. When the channel is full and a new message
/// is sent, the oldest message is dropped. Slow subscribers may miss messages if they
/// can't keep up with the broadcast rate.
///
/// # Cloning
///
/// The broadcaster can be cloned cheaply (Arc-based internally). All clones share
/// the same broadcast channel.
#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<Message>,
    capacity: usize,
}

impl Broadcaster {
    /// Create new broadcaster with specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of messages to buffer. When full, oldest messages
    ///   are dropped. Typical values: 16-256 depending on message rate and subscriber speed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::Broadcaster;
    ///
    /// // Small capacity for low-traffic scenarios
    /// let broadcaster = Broadcaster::new(16);
    ///
    /// // Larger capacity for high-traffic scenarios
    /// let broadcaster = Broadcaster::new(256);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx, capacity }
    }

    /// Broadcast message to all subscribers
    ///
    /// Sends a message to all active subscribers. Each subscriber receives a clone
    /// of the message. If no subscribers exist, the message is dropped.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there are no active subscribers. This is usually not an error
    /// condition - it just means no one is listening.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::{Broadcaster, Message};
    ///
    /// # async fn example() {
    /// let broadcaster = Broadcaster::new(100);
    /// let mut subscriber = broadcaster.subscribe();
    ///
    /// // Broadcast succeeds when there are subscribers
    /// broadcaster.broadcast(Message::text("Hello")).ok();
    ///
    /// assert_eq!(subscriber.recv().await.unwrap(), Message::text("Hello"));
    /// # }
    /// ```
    pub fn broadcast(&self, msg: Message) -> Result<usize, broadcast::error::SendError<Message>> {
        self.tx.send(msg)
    }

    /// Subscribe to broadcasts
    ///
    /// Creates a new receiver that will receive all messages broadcast after subscription.
    /// Messages sent before subscription are not received.
    ///
    /// # Returns
    ///
    /// A `broadcast::Receiver<Message>` that can be used to receive messages.
    /// Use `.recv().await` to receive the next message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::{Broadcaster, Message};
    ///
    /// # async fn example() {
    /// let broadcaster = Broadcaster::new(100);
    ///
    /// // Subscribe before broadcasting
    /// let mut subscriber = broadcaster.subscribe();
    ///
    /// broadcaster.broadcast(Message::text("Hello")).ok();
    ///
    /// // Receive the message
    /// let msg = subscriber.recv().await.unwrap();
    /// assert_eq!(msg, Message::text("Hello"));
    /// # }
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }

    /// Get the number of active subscribers
    ///
    /// Returns the current number of receivers subscribed to this broadcaster.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::Broadcaster;
    ///
    /// let broadcaster = Broadcaster::new(100);
    /// assert_eq!(broadcaster.subscriber_count(), 0);
    ///
    /// let _sub1 = broadcaster.subscribe();
    /// assert_eq!(broadcaster.subscriber_count(), 1);
    ///
    /// let _sub2 = broadcaster.subscribe();
    /// assert_eq!(broadcaster.subscriber_count(), 2);
    /// ```
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Check if there are any active subscribers
    ///
    /// Returns `true` if at least one subscriber exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::Broadcaster;
    ///
    /// let broadcaster = Broadcaster::new(100);
    /// assert!(!broadcaster.has_subscribers());
    ///
    /// let _subscriber = broadcaster.subscribe();
    /// assert!(broadcaster.has_subscribers());
    /// ```
    pub fn has_subscribers(&self) -> bool {
        self.tx.receiver_count() > 0
    }

    /// Get the channel capacity
    ///
    /// Returns the maximum number of messages that can be buffered.
    /// This is stored separately since Tokio's broadcast::Sender doesn't expose capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ruxno_clean::upgrade::websocket::Broadcaster;
    ///
    /// let broadcaster = Broadcaster::new(100);
    /// assert_eq!(broadcaster.capacity(), 100);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for Broadcaster {
    /// Create broadcaster with default capacity of 64 messages
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_new() {
        let broadcaster = Broadcaster::new(100);
        assert_eq!(broadcaster.capacity(), 100);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn test_broadcaster_default() {
        let broadcaster = Broadcaster::default();
        assert_eq!(broadcaster.capacity(), 64);
    }

    #[test]
    fn test_broadcaster_subscribe() {
        let broadcaster = Broadcaster::new(100);
        assert_eq!(broadcaster.subscriber_count(), 0);

        let _sub1 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);

        let _sub2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);
    }

    #[tokio::test]
    async fn test_broadcast_to_single_subscriber() {
        let broadcaster = Broadcaster::new(100);
        let mut subscriber = broadcaster.subscribe();

        let msg = Message::text("Hello");
        broadcaster.broadcast(msg.clone()).unwrap();

        let received = subscriber.recv().await.unwrap();
        assert_eq!(received, msg);
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_subscribers() {
        let broadcaster = Broadcaster::new(100);
        let mut sub1 = broadcaster.subscribe();
        let mut sub2 = broadcaster.subscribe();
        let mut sub3 = broadcaster.subscribe();

        let msg = Message::text("Broadcast message");
        broadcaster.broadcast(msg.clone()).unwrap();

        assert_eq!(sub1.recv().await.unwrap(), msg);
        assert_eq!(sub2.recv().await.unwrap(), msg);
        assert_eq!(sub3.recv().await.unwrap(), msg);
    }

    #[tokio::test]
    async fn test_broadcast_no_subscribers() {
        let broadcaster = Broadcaster::new(100);

        let msg = Message::text("No one listening");
        let result = broadcaster.broadcast(msg);

        // Broadcasting with no subscribers returns an error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_multiple_messages() {
        let broadcaster = Broadcaster::new(100);
        let mut subscriber = broadcaster.subscribe();

        broadcaster.broadcast(Message::text("First")).unwrap();
        broadcaster.broadcast(Message::text("Second")).unwrap();
        broadcaster.broadcast(Message::text("Third")).unwrap();

        assert_eq!(subscriber.recv().await.unwrap(), Message::text("First"));
        assert_eq!(subscriber.recv().await.unwrap(), Message::text("Second"));
        assert_eq!(subscriber.recv().await.unwrap(), Message::text("Third"));
    }

    #[tokio::test]
    async fn test_late_subscriber_misses_old_messages() {
        let broadcaster = Broadcaster::new(100);
        let mut early_sub = broadcaster.subscribe();

        // Send message before late subscriber joins
        broadcaster
            .broadcast(Message::text("Early message"))
            .unwrap();

        let mut late_sub = broadcaster.subscribe();

        // Send message after late subscriber joins
        broadcaster
            .broadcast(Message::text("Late message"))
            .unwrap();

        // Early subscriber gets both messages
        assert_eq!(
            early_sub.recv().await.unwrap(),
            Message::text("Early message")
        );
        assert_eq!(
            early_sub.recv().await.unwrap(),
            Message::text("Late message")
        );

        // Late subscriber only gets the second message
        assert_eq!(
            late_sub.recv().await.unwrap(),
            Message::text("Late message")
        );
    }

    #[tokio::test]
    async fn test_subscriber_drop_reduces_count() {
        let broadcaster = Broadcaster::new(100);

        let sub1 = broadcaster.subscribe();
        let sub2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);

        drop(sub1);
        assert_eq!(broadcaster.subscriber_count(), 1);

        drop(sub2);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn test_has_subscribers() {
        let broadcaster = Broadcaster::new(100);
        assert!(!broadcaster.has_subscribers());

        let _sub = broadcaster.subscribe();
        assert!(broadcaster.has_subscribers());
    }

    #[test]
    fn test_broadcaster_clone() {
        let broadcaster = Broadcaster::new(100);
        let clone = broadcaster.clone();

        // Both share the same channel
        let _sub1 = broadcaster.subscribe();
        let _sub2 = clone.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 2);
        assert_eq!(clone.subscriber_count(), 2);
    }

    #[tokio::test]
    async fn test_broadcast_binary_message() {
        let broadcaster = Broadcaster::new(100);
        let mut subscriber = broadcaster.subscribe();

        let data = vec![1, 2, 3, 4, 5];
        let msg = Message::binary(data.clone());
        broadcaster.broadcast(msg.clone()).unwrap();

        let received = subscriber.recv().await.unwrap();
        assert_eq!(received, msg);
    }

    #[tokio::test]
    async fn test_broadcast_control_messages() {
        let broadcaster = Broadcaster::new(100);
        let mut subscriber = broadcaster.subscribe();

        broadcaster.broadcast(Message::ping(vec![1, 2, 3])).unwrap();
        broadcaster.broadcast(Message::pong(vec![4, 5, 6])).unwrap();
        broadcaster.broadcast(Message::close()).unwrap();

        assert_eq!(
            subscriber.recv().await.unwrap(),
            Message::ping(vec![1, 2, 3])
        );
        assert_eq!(
            subscriber.recv().await.unwrap(),
            Message::pong(vec![4, 5, 6])
        );
        assert_eq!(subscriber.recv().await.unwrap(), Message::close());
    }

    #[tokio::test]
    async fn test_capacity_overflow() {
        // Small capacity to test overflow behavior
        let broadcaster = Broadcaster::new(2);
        let mut subscriber = broadcaster.subscribe();

        // Fill the channel
        broadcaster.broadcast(Message::text("1")).unwrap();
        broadcaster.broadcast(Message::text("2")).unwrap();

        // This will cause the oldest message to be dropped for slow subscribers
        broadcaster.broadcast(Message::text("3")).unwrap();

        // Subscriber may receive a lagged error if it can't keep up
        // This is expected behavior for broadcast channels
        match subscriber.recv().await {
            Ok(msg) => {
                // If we get a message, it should be one of the sent messages
                assert!(
                    msg == Message::text("1")
                        || msg == Message::text("2")
                        || msg == Message::text("3")
                );
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // Lagged error is expected when capacity is exceeded
                assert!(n > 0, "Should report number of missed messages");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_broadcaster_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Broadcaster>();
        assert_sync::<Broadcaster>();
    }
}
