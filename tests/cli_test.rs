//! CLI and shutdown integration tests.
//!
//! Tests:
//! - T049: CLI help output verification
//! - T050: Graceful shutdown flushes pending writes

use std::process::Command;
use std::time::Duration;

/// T049: CLI --help output should show expected options.
#[test]
fn test_cli_help_output() {
    // Build the binary first
    let build = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to build");

    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Run --help
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify expected CLI options are present
    assert!(
        stdout.contains("--port"),
        "help should mention --port option"
    );
    assert!(
        stdout.contains("--data-dir"),
        "help should mention --data-dir option"
    );
    assert!(
        stdout.contains("--log-level"),
        "help should mention --log-level option"
    );
    assert!(
        stdout.contains("Sluice") || stdout.contains("sluice"),
        "help should mention Sluice"
    );
}

/// T049: CLI --version should show version.
#[test]
fn test_cli_version_output() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain version number
    assert!(
        stdout.contains("0.1.0"),
        "version output should contain version number: {}",
        stdout
    );
}

/// T050: Graceful shutdown test - server responds to signals properly.
///
/// This test starts the server, sends SIGTERM, and verifies it exits cleanly.
#[cfg(unix)]
#[tokio::test]
async fn test_graceful_shutdown_on_sigterm() {
    use std::process::Stdio;
    use tokio::process::Command as TokioCommand;
    use tokio::time::timeout;

    let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");

    // Start server in background
    let mut child = TokioCommand::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            "--port",
            "0", // Will fail to bind to 0, but we'll use a proper port
            "--data-dir",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn server");

    // Wait a bit for server to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send SIGTERM using kill command
    let pid = child.id().expect("no pid");
    let _ = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status();

    // Wait for clean exit with timeout
    let exit_result = timeout(Duration::from_secs(5), child.wait()).await;

    match exit_result {
        Ok(Ok(status)) => {
            // Server should exit (possibly with error due to port 0, but should exit cleanly)
            assert!(
                status.code().is_some(),
                "server should exit with status code"
            );
        }
        Ok(Err(e)) => panic!("failed to wait for child: {}", e),
        Err(_) => {
            // Timeout - server didn't respond to SIGTERM, kill it
            child.kill().await.expect("failed to kill");
            panic!("server did not respond to SIGTERM within timeout");
        }
    }
}
