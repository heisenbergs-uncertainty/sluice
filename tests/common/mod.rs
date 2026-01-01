//! Test utilities and server harness for Sluice tests.
//!
//! Provides:
//! - In-process test server setup
//! - gRPC client helpers
//! - Test database fixtures

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Test fixture that manages a temporary database directory.
///
/// The directory is automatically cleaned up when the fixture is dropped.
pub struct TestFixture {
    /// Temporary directory for test database
    pub temp_dir: TempDir,
    /// Path to the database file
    pub db_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with a temporary database directory.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        Self { temp_dir, db_path }
    }

    /// Get the database path as a string.
    pub fn db_path_str(&self) -> &str {
        self.db_path.to_str().expect("invalid path")
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for a condition to become true with timeout.
///
/// # Arguments
///
/// * `timeout` - Maximum time to wait
/// * `condition` - Closure that returns true when condition is met
///
/// # Returns
///
/// `true` if condition was met, `false` if timeout expired
pub async fn wait_for<F>(timeout: std::time::Duration, mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creates_temp_dir() {
        let fixture = TestFixture::new();
        assert!(fixture.temp_dir.path().exists());
        assert!(fixture.db_path_str().contains("test.db"));
    }
}

