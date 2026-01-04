//! Connection registry for consumer group takeover.
//!
//! Tracks active consumers per (topic_id, consumer_group) to support
//! seamless takeover when a new consumer connects with the same group.

use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;

/// Key for identifying a unique consumer group connection.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ConsumerGroupKey {
    pub topic_id: i64,
    pub consumer_group: String,
}

/// Registry tracking active consumer connections.
///
/// When a new consumer connects to an existing consumer group,
/// the prior connection is terminated with ABORTED status.
#[derive(Debug, Default)]
pub struct ConnectionRegistry {
    /// Map of active connections: key -> cancellation sender
    active: Mutex<HashMap<ConsumerGroupKey, oneshot::Sender<()>>>,
}

impl ConnectionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new consumer connection.
    ///
    /// Returns a receiver that will be signaled when this connection
    /// should be terminated (due to takeover by another consumer).
    ///
    /// If there's already an active connection for this consumer group,
    /// it will be terminated immediately.
    pub fn register(&self, key: ConsumerGroupKey) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();

        let mut active = self.active.lock().unwrap();

        // If there's an existing connection, terminate it
        if let Some(old_tx) = active.remove(&key) {
            tracing::info!(
                topic_id = key.topic_id,
                consumer_group = %key.consumer_group,
                "Terminating prior consumer connection (takeover)"
            );
            // Send termination signal (ignore if receiver already dropped)
            let _ = old_tx.send(());
        }

        // Register new connection
        active.insert(key, tx);

        rx
    }

    /// Unregister a consumer connection.
    ///
    /// Called when a consumer disconnects normally.
    pub fn unregister(&self, key: &ConsumerGroupKey) {
        let mut active = self.active.lock().unwrap();
        active.remove(key);
    }

    /// Get the number of active connections.
    #[cfg(test)]
    pub fn active_count(&self) -> usize {
        self.active.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unregister() {
        let registry = ConnectionRegistry::new();
        let key = ConsumerGroupKey {
            topic_id: 1,
            consumer_group: "test".to_string(),
        };

        let _rx = registry.register(key.clone());
        assert_eq!(registry.active_count(), 1);

        registry.unregister(&key);
        assert_eq!(registry.active_count(), 0);
    }

    #[tokio::test]
    async fn test_takeover_signals_prior_connection() {
        let registry = ConnectionRegistry::new();
        let key = ConsumerGroupKey {
            topic_id: 1,
            consumer_group: "workers".to_string(),
        };

        // First consumer registers
        let rx1 = registry.register(key.clone());

        // Second consumer registers with same key (takeover)
        let _rx2 = registry.register(key.clone());

        // First consumer should receive termination signal
        // (recv returns Ok(()) when sender sends, Err when sender dropped)
        let result = rx1.await;
        assert!(
            result.is_ok(),
            "Prior connection should receive termination signal"
        );

        assert_eq!(registry.active_count(), 1);
    }

    #[tokio::test]
    async fn test_different_groups_independent() {
        let registry = ConnectionRegistry::new();
        let key1 = ConsumerGroupKey {
            topic_id: 1,
            consumer_group: "group-a".to_string(),
        };
        let key2 = ConsumerGroupKey {
            topic_id: 1,
            consumer_group: "group-b".to_string(),
        };

        let _rx1 = registry.register(key1.clone());
        let _rx2 = registry.register(key2.clone());

        assert_eq!(registry.active_count(), 2);
    }

    #[tokio::test]
    async fn test_different_topics_independent() {
        let registry = ConnectionRegistry::new();
        let key1 = ConsumerGroupKey {
            topic_id: 1,
            consumer_group: "workers".to_string(),
        };
        let key2 = ConsumerGroupKey {
            topic_id: 2,
            consumer_group: "workers".to_string(),
        };

        let _rx1 = registry.register(key1.clone());
        let _rx2 = registry.register(key2.clone());

        assert_eq!(registry.active_count(), 2);
    }
}
