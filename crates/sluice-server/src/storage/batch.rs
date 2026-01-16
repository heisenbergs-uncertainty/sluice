//! Group commit batch logic for high-throughput writes.
//!
//! Implements batching per research.md decision 1:
//! - Collect multiple writes into single transaction
//! - Amortize fsync cost across batch
//! - Target: 5,000+ msg/s throughput

use std::time::{Duration, Instant};

/// Configuration for batch commits.
#[derive(Debug, Clone, Copy)]
pub struct BatchConfig {
    /// Maximum number of messages in a batch
    pub max_batch_size: usize,
    /// Maximum time to wait for a full batch
    pub max_batch_delay: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_delay: Duration::from_millis(5),
        }
    }
}

impl BatchConfig {
    /// Create a BatchConfig from application config values.
    pub fn from_config(batch_size: usize, batch_delay_ms: u64) -> Self {
        Self {
            max_batch_size: batch_size,
            max_batch_delay: Duration::from_millis(batch_delay_ms),
        }
    }

    /// Create a test config with small batch size and short delay.
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            max_batch_size: 1,
            max_batch_delay: Duration::from_millis(1),
        }
    }
}

/// Batch accumulator for write operations.
///
/// Collects operations until either:
/// - The batch is full (max_batch_size)
/// - The timeout expires (max_batch_delay)
#[derive(Debug)]
pub struct BatchAccumulator<T> {
    config: BatchConfig,
    items: Vec<T>,
    batch_start: Option<Instant>,
}

impl<T> BatchAccumulator<T> {
    /// Create a new batch accumulator with the given configuration.
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            items: Vec::with_capacity(config.max_batch_size),
            batch_start: None,
        }
    }

    /// Add an item to the batch.
    ///
    /// Returns true if the batch is now ready to flush.
    pub fn push(&mut self, item: T) -> bool {
        if self.batch_start.is_none() {
            self.batch_start = Some(Instant::now());
        }
        self.items.push(item);
        self.is_ready()
    }

    /// Check if the batch is ready to flush.
    pub fn is_ready(&self) -> bool {
        if self.items.is_empty() {
            return false;
        }

        // Size-based trigger
        if self.items.len() >= self.config.max_batch_size {
            return true;
        }

        // Time-based trigger
        if let Some(start) = self.batch_start {
            if start.elapsed() >= self.config.max_batch_delay {
                return true;
            }
        }

        false
    }

    /// Time remaining until the batch should be flushed.
    ///
    /// Returns None if the batch is empty or already ready.
    pub fn time_until_ready(&self) -> Option<Duration> {
        if self.items.is_empty() {
            return None;
        }

        if self.is_ready() {
            return Some(Duration::ZERO);
        }

        self.batch_start.map(|start| {
            let elapsed = start.elapsed();
            self.config.max_batch_delay.saturating_sub(elapsed)
        })
    }

    /// Drain the batch, returning all accumulated items.
    pub fn drain(&mut self) -> Vec<T> {
        self.batch_start = None;
        std::mem::replace(
            &mut self.items,
            Vec::with_capacity(self.config.max_batch_size),
        )
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the current batch size.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size_trigger() {
        let config = BatchConfig {
            max_batch_size: 3,
            max_batch_delay: Duration::from_secs(10),
        };
        let mut batch = BatchAccumulator::new(config);

        assert!(!batch.push(1));
        assert!(!batch.push(2));
        assert!(batch.push(3)); // Now ready

        let items = batch.drain();
        assert_eq!(items, vec![1, 2, 3]);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_time_trigger() {
        let config = BatchConfig {
            max_batch_size: 100,
            max_batch_delay: Duration::from_millis(10),
        };
        let mut batch = BatchAccumulator::new(config);

        batch.push(1);
        assert!(!batch.is_ready());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(15));
        assert!(batch.is_ready());
    }
}
