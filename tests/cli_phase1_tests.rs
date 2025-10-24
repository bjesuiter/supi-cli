// Phase 1: Basic Process Spawning
// Tests for CLI argument parsing and basic process management

mod cli_test_utils;

use assert_cmd::Command;
use cli_test_utils::create_pty_with_reader;
use portable_pty::CommandBuilder;
use predicates::prelude::*;
use std::time::Duration;

// ============================================================================
// CLI Tests
// ============================================================================

// Manual test: cargo run -- --help
#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("supi").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A lightweight process supervisor with restart capabilities",
        ))
        .stdout(predicate::str::contains("Usage:"));
}

// Manual test: cargo run -- --version
#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("supi").unwrap();
    let cargo_pkg_version = env!("CARGO_PKG_VERSION");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(cargo_pkg_version));
}

// Manual test: cargo run -- -V
#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("supi").unwrap();
    let cargo_pkg_version = env!("CARGO_PKG_VERSION");
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(cargo_pkg_version));
}

// Manual test: cargo run --
#[test]
fn test_missing_command_fails() {
    let mut cmd = Command::cargo_bin("supi").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// Manual test: cargo run -- this_command_does_not_exist_xyz123
#[test]
fn test_nonexistent_command() {
    let mut cmd = Command::cargo_bin("supi").unwrap();
    cmd.arg("this_command_does_not_exist_xyz123")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to spawn"));
}

// ============================================================================
// Process Tests
// ============================================================================

// Manual test: cargo run -- --stop-on-child-exit echo "hello world"
#[test]
fn test_simple_echo() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.args(&["--stop-on-child-exit", "echo", "hello world"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    assert!(
        output_str.contains("hello world"),
        "Expected 'hello world' in output. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --stop-on-child-exit echo "test message"
#[test]
fn test_stop_on_child_exit_flag() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.args(&["--stop-on-child-exit", "echo", "test message"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    assert!(
        output_str.contains("test message"),
        "Expected 'test message' in output. Output:\n{}",
        output_str
    );
    assert!(
        output_str.contains("Exiting (--stop-on-child-exit is set)"),
        "Expected exit message. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --stop-on-child-exit bash -- -c "echo line1 && echo line2 && echo line3"
#[test]
fn test_stdout_forwarding() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.args(&[
        "--stop-on-child-exit",
        "--",
        "bash",
        "-c",
        "echo line1 && echo line2 && echo line3",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    assert!(
        output_str.contains("line1"),
        "Expected 'line1' in output. Output:\n{}",
        output_str
    );
    assert!(
        output_str.contains("line2"),
        "Expected 'line2' in output. Output:\n{}",
        output_str
    );
    assert!(
        output_str.contains("line3"),
        "Expected 'line3' in output. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --stop-on-child-exit bash -- -c "echo 'error message' >&2"
#[test]
fn test_stderr_forwarding() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.args(&[
        "--stop-on-child-exit",
        "--",
        "bash",
        "-c",
        "echo 'error message' >&2",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // In PTY, stderr and stdout are merged into the PTY output
    assert!(
        output_str.contains("error message"),
        "Expected 'error message' in output. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}
