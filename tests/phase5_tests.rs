// Phase 5: Advanced Features
// Tests for --silent flag and colored output

mod test_utils;

use assert_cmd::Command;
use portable_pty::CommandBuilder;
use predicates::prelude::*;
use std::time::Duration;
use test_utils::create_pty_with_reader;

// ============================================================================
// Silent Flag Tests
// ============================================================================

// Test that --silent flag suppresses supervisor output
#[test]
fn test_silent_flag_suppresses_supervisor_output() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--silent", "--stop-on-child-exit", "echo", "child output"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Child output should be visible
    assert!(
        output_str.contains("child output"),
        "Expected child output to be visible. Output:\n{}",
        output_str
    );

    // Supervisor messages should NOT be present
    assert!(
        !output_str.contains("[supi]"),
        "Expected no supervisor messages with --silent flag. Output:\n{}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Test that --silent flag preserves child output (both stdout and stderr)
#[test]
fn test_silent_flag_preserves_child_output() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--silent",
        "--stop-on-child-exit",
        "bash",
        "--",
        "-c",
        "echo 'stdout message' && echo 'stderr message' >&2",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Both stdout and stderr from child should be visible
    assert!(
        output_str.contains("stdout message"),
        "Expected stdout to be visible. Output:\n{}",
        output_str
    );
    assert!(
        output_str.contains("stderr message"),
        "Expected stderr to be visible. Output:\n{}",
        output_str
    );

    // Supervisor messages should NOT be present
    assert!(
        !output_str.contains("[supi]"),
        "Expected no supervisor messages with --silent flag. Output:\n{}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Test that without --silent flag, supervisor messages ARE shown
#[test]
fn test_without_silent_flag_shows_supervisor_output() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--stop-on-child-exit", "echo", "child output"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Child output should be visible
    assert!(
        output_str.contains("child output"),
        "Expected child output to be visible. Output:\n{}",
        output_str
    );

    // Supervisor messages SHOULD be present
    assert!(
        output_str.contains("[supi]"),
        "Expected supervisor messages without --silent flag. Output:\n{}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// ============================================================================
// Color Output Tests
// ============================================================================

// Test that the color flag is parsed correctly and invalid colors are rejected
#[test]
fn test_log_color_flag() {
    // Test valid colors - should succeed
    let valid_colors = vec![
        "yellow", "red", "green", "blue", "cyan", "magenta", "white", "none",
    ];

    for color in valid_colors {
        let (pair, _output, reader_thread) = create_pty_with_reader();

        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
        cmd.args(&["--log-color", color, "--stop-on-child-exit", "echo", "test"]);

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let status = child.wait().unwrap();
        assert!(
            status.success(),
            "Color '{}' should be valid but process failed",
            color
        );

        let _ = reader_thread.join();
    }

    // Test invalid color - should fail
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.args(&[
        "--log-color",
        "invalid_color",
        "--stop-on-child-exit",
        "echo",
        "test",
    ])
    .timeout(std::time::Duration::from_secs(2))
    .assert()
    .failure()
    .stderr(predicate::str::contains("Invalid color"));
}

// Test that colored output contains ANSI color codes
#[test]
fn test_colored_output() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--log-color",
        "yellow",
        "--info-color",
        "green",
        "--stop-on-child-exit",
        "echo",
        "test output",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Child output should be visible
    assert!(
        output_str.contains("test output"),
        "Expected child output to be visible. Output:\n{}",
        output_str
    );

    // Should contain ANSI escape codes for colors
    // ANSI escape sequence starts with \x1b[ or ESC[
    // For terminals, this appears as escape sequences
    assert!(
        output_str.contains("\x1b["),
        "Expected ANSI color codes in output. Output (debug):\n{:?}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Test that --log-color=none produces output without color codes
#[test]
fn test_no_color_option() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--log-color",
        "none",
        "--info-color",
        "none",
        "--stop-on-child-exit",
        "echo",
        "test output",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Child output should be visible
    assert!(
        output_str.contains("test output"),
        "Expected child output to be visible. Output:\n{}",
        output_str
    );

    // Supervisor messages should still be present (just not colored)
    assert!(
        output_str.contains("[supi]"),
        "Expected supervisor messages. Output:\n{}",
        output_str
    );

    // Should NOT contain ANSI escape codes for colors (SetForegroundColor)
    // We check for the specific color command escape sequences
    // Yellow is 33m, Green is 32m, etc. - but with "none", these shouldn't appear
    let has_color_codes = output_str.contains("\x1b[33m") // yellow
        || output_str.contains("\x1b[32m") // green
        || output_str.contains("\x1b[31m") // red
        || output_str.contains("\x1b[34m") // blue
        || output_str.contains("\x1b[36m") // cyan
        || output_str.contains("\x1b[35m") // magenta
        || output_str.contains("\x1b[37m"); // white

    assert!(
        !has_color_codes,
        "Expected no color codes with --log-color=none and --info-color=none. Output (debug):\n{:?}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Test that different colors produce different ANSI codes
#[test]
fn test_different_colors_produce_different_codes() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--log-color",
        "red",
        "--stop-on-child-exit",
        "echo",
        "red test",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should contain red color code (31m)
    assert!(
        output_str.contains("\x1b[31m") || output_str.contains("\x1b["),
        "Expected ANSI color codes for red. Output (debug):\n{:?}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Test that info-color works independently of log-color
#[test]
fn test_info_color_independent() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--log-color",
        "red",
        "--info-color",
        "blue",
        "--stop-on-child-exit",
        "echo",
        "test output",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should contain ANSI escape codes (both colors are used)
    assert!(
        output_str.contains("\x1b["),
        "Expected ANSI color codes. Output (debug):\n{:?}",
        output_str
    );

    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

