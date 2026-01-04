//! Notification bus for subscription wake-up.
//!
//! Per research.md decision 3:
//! - tokio::sync::broadcast for pub-sub notifications
//! - Lightweight notifications trigger targeted fetches

use tokio::sync::broadcast::{self, Receiver, Sender};

/// Notification sent when new data is available for a topic.
#[derive(Debug, Clone, Copy)]
pub struct NewDataNotification {
    /// The topic that has new data
    pub topic_id: i64,
    /// The maximum sequence number available
    pub max_seq: i64,
}

/// Notification bus for waking sleeping subscriptions.
///
/// Uses tokio::sync::broadcast for efficient pub-sub notifications.
/// Subscriptions register to receive notifications and wake when
/// new data is available for their topic.
#[derive(Clone)]
pub struct NotificationBus {
    sender: Sender<NewDataNotification>,
}

impl NotificationBus {
    /// Create a new notification bus with the given capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of buffered notifications.
    ///   Older notifications are dropped if consumers fall behind.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to notifications.
    ///
    /// Returns a receiver that will receive all future notifications.
    pub fn subscribe(&self) -> Receiver<NewDataNotification> {
        self.sender.subscribe()
    }

    /// Notify subscribers that new data is available.
    ///
    /// This is called by the writer thread after committing a batch.
    ///
    /// # Arguments
    ///
    /// * `topic_id` - The topic with new data
    /// * `max_seq` - The maximum sequence number now available
    ///
    /// # Returns
    ///
    /// The number of receivers that received the notification.
    pub fn notify(&self, topic_id: i64, max_seq: i64) -> usize {
        // send() returns an error if there are no receivers, which is fine
        self.sender
            .send(NewDataNotification { topic_id, max_seq })
            .unwrap_or(0)
    }

    /// Get the number of active receivers.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for NotificationBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_bus() {
        let bus = NotificationBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        // Send notification
        let count = bus.notify(1, 42);
        assert_eq!(count, 2);

        // Both receivers get it
        let n1 = rx1.recv().await.unwrap();
        assert_eq!(n1.topic_id, 1);
        assert_eq!(n1.max_seq, 42);

        let n2 = rx2.recv().await.unwrap();
        assert_eq!(n2.topic_id, 1);
        assert_eq!(n2.max_seq, 42);
    }

    #[test]
    fn test_notification_without_receivers() {
        let bus = NotificationBus::new(16);

        // No receivers - should not panic
        let count = bus.notify(1, 42);
        assert_eq!(count, 0);
    }
}
