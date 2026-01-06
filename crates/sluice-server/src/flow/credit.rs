//! Credit-based flow control implementation.
//!
//! Per research.md decision 4:
//! - AtomicU32 for lock-free credit accounting
//! - Split inbound/outbound handlers to avoid deadlock

use std::sync::atomic::{AtomicU32, Ordering};

/// Credit balance for a subscription.
///
/// Uses atomic operations for thread-safe credit management.
/// Credits control the flow of messages from server to client,
/// implementing backpressure to prevent overwhelming slow consumers.
#[derive(Debug)]
pub struct CreditBalance {
    credits: AtomicU32,
}

impl Default for CreditBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl CreditBalance {
    /// Create a new credit balance starting at 0.
    pub fn new() -> Self {
        Self {
            credits: AtomicU32::new(0),
        }
    }

    /// Create a new credit balance with initial credits.
    pub fn with_initial(initial: u32) -> Self {
        Self {
            credits: AtomicU32::new(initial),
        }
    }

    /// Add credits to the balance.
    ///
    /// Returns the new total.
    pub fn add(&self, amount: u32) -> u32 {
        self.credits.fetch_add(amount, Ordering::SeqCst) + amount
    }

    /// Try to consume one credit.
    ///
    /// Returns true if a credit was consumed, false if no credits available.
    pub fn try_consume(&self) -> bool {
        loop {
            let current = self.credits.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }
            if self
                .credits
                .compare_exchange(current, current - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
            // CAS failed, retry
        }
    }

    /// Try to consume multiple credits.
    ///
    /// Returns the number of credits actually consumed (may be less than requested).
    pub fn try_consume_many(&self, amount: u32) -> u32 {
        loop {
            let current = self.credits.load(Ordering::SeqCst);
            if current == 0 {
                return 0;
            }
            let to_consume = current.min(amount);
            if self
                .credits
                .compare_exchange(
                    current,
                    current - to_consume,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                return to_consume;
            }
            // CAS failed, retry
        }
    }

    /// Get the current credit count.
    pub fn available(&self) -> u32 {
        self.credits.load(Ordering::SeqCst)
    }

    /// Reset credits to zero.
    ///
    /// Returns the previous credit count.
    pub fn reset(&self) -> u32 {
        self.credits.swap(0, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_at_zero() {
        let balance = CreditBalance::new();
        assert_eq!(balance.available(), 0);
    }

    #[test]
    fn test_with_initial() {
        let balance = CreditBalance::with_initial(10);
        assert_eq!(balance.available(), 10);
    }

    #[test]
    fn test_add_credits() {
        let balance = CreditBalance::new();
        assert_eq!(balance.add(5), 5);
        assert_eq!(balance.add(3), 8);
        assert_eq!(balance.available(), 8);
    }

    #[test]
    fn test_try_consume_success() {
        let balance = CreditBalance::with_initial(2);
        assert!(balance.try_consume());
        assert_eq!(balance.available(), 1);
        assert!(balance.try_consume());
        assert_eq!(balance.available(), 0);
    }

    #[test]
    fn test_try_consume_empty() {
        let balance = CreditBalance::new();
        assert!(!balance.try_consume());
        assert_eq!(balance.available(), 0);
    }

    #[test]
    fn test_try_consume_many() {
        let balance = CreditBalance::with_initial(5);
        assert_eq!(balance.try_consume_many(3), 3);
        assert_eq!(balance.available(), 2);
        assert_eq!(balance.try_consume_many(5), 2);
        assert_eq!(balance.available(), 0);
    }

    #[test]
    fn test_reset() {
        let balance = CreditBalance::with_initial(10);
        assert_eq!(balance.reset(), 10);
        assert_eq!(balance.available(), 0);
    }
}
