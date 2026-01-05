use std::time::Duration;

#[allow(dead_code)]
pub(crate) fn reconnect_backoff(attempt: u32) -> Duration {
    // Exponential backoff starting at 100ms, capped at 5s.
    let base_ms: u64 = 100;
    let cap_ms: u64 = 5_000;
    let exp = attempt.min(16);
    let factor = 1u64.checked_shl(exp).unwrap_or(u64::MAX);
    let delay_ms = base_ms.saturating_mul(factor);
    Duration::from_millis(delay_ms.min(cap_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases_and_caps() {
        let d0 = reconnect_backoff(0);
        let d1 = reconnect_backoff(1);
        let d2 = reconnect_backoff(2);
        assert!(d0 < d1);
        assert!(d1 < d2);

        let capped = reconnect_backoff(999);
        assert_eq!(capped, Duration::from_millis(5_000));
    }
}
