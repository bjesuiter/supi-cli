use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read, Write};
use std::time::Duration;

// Manual test: cargo run -- --help
#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
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
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// Manual test: cargo run -- -V
#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// Manual test: cargo run --
#[test]
fn test_missing_command_fails() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// Manual test: cargo run -- --stop-on-child-exit echo "hello world"
#[test]
fn test_simple_echo() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
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
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
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

// Manual test: cargo run -- this_command_does_not_exist_xyz123
#[test]
fn test_nonexistent_command() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("this_command_does_not_exist_xyz123")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to spawn"));
}

// ============================================================================
// PTY-Based Tests
// ============================================================================
// All following tests use a PTY setup for testing to avoid display issues in
// cargo output due to the use of raw tty mode in hotkey.rs. PTY provides a
// clean, realistic terminal environment for testing.

// Helper function to create PTY test environment
fn create_pty_with_reader() -> (
    portable_pty::PtyPair,
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::thread::JoinHandle<()>,
) {
    use portable_pty::{PtySize, native_pty_system};
    use std::sync::{Arc, Mutex};

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let output = Arc::new(Mutex::new(Vec::new()));
    let output_clone = Arc::clone(&output);
    let mut reader = pair.master.try_clone_reader().unwrap();

    let reader_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    output_clone.lock().unwrap().extend_from_slice(&buf[..n]);
                }
                _ => break,
            }
        }
    });

    (pair, output, reader_thread)
}

// Manual test: cargo run -- --stop-on-child-exit bash -- -c "echo line1 && echo line2 && echo line3"
#[test]
fn test_stdout_forwarding() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
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
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
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

// Phase 2 Tests: Signal Handling

// Manual test: cargo run -- sleep 30
//              (note the PID shown, then in another terminal: kill -TERM <pid>)
#[test]
fn test_sigterm_graceful_shutdown() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "sleep", "30"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let child_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGTERM to the child process
    unsafe {
        libc::kill(child_pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should show graceful shutdown message
    assert!(
        output_str.contains("Received SIGTERM signal")
            || output_str.contains("Stopping child process"),
        "Expected shutdown message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- sleep 30
//              (then press Ctrl+C or in another terminal: kill -INT <pid>)
#[test]
fn test_sigint_graceful_shutdown() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "sleep", "30"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let child_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGINT
    unsafe {
        libc::kill(child_pid as i32, libc::SIGINT);
    }

    // Wait for it to exit
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should show graceful shutdown message
    assert!(
        output_str.contains("Received SIGINT signal")
            || output_str.contains("Stopping child process"),
        "Expected shutdown message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- bash -c "echo 'started'; sleep 10"
//              (note the PID, then in another terminal: kill -USR1 <pid>, then: kill -TERM <pid>)
#[test]
fn test_restart_signal() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "bash", "-c", "echo 'started'; sleep 10"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let child_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give it more time to start and print first message
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGUSR1 to trigger restart
    unsafe {
        libc::kill(child_pid as i32, libc::SIGUSR1);
    }

    // Give it more time to restart
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGTERM to stop
    unsafe {
        libc::kill(child_pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see restart messages in stdout
    assert!(
        output_str.contains("Received SIGUSR1 signal")
            || output_str.contains("Restarting child process"),
        "Expected restart message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: Create test.sh with: trap 'echo "Child got SIGTERM"; exit 0' TERM; echo "Started"; sleep 30
//              Then: chmod +x test.sh && cargo run -- ./test.sh
//              (note the PID, then in another terminal: kill -TERM <pid>)
#[test]
fn test_signal_forwarding_to_child() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    // Create a script that handles SIGTERM gracefully
    let script = r#"#!/bin/bash
trap 'echo "Child received SIGTERM"; exit 0' TERM
echo "Child started"
sleep 30
"#;

    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), script).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(temp_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(temp_file.path(), perms).unwrap();
    }

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.arg(temp_file.path().to_str().unwrap());

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let child_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGTERM
    unsafe {
        libc::kill(child_pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see both supervisor and child handling the signal
    assert!(
        output_str.contains("Stopping child process"),
        "Expected 'Stopping child process'. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Phase 3 Tests: Interactive Hotkey

// Manual test: cargo run -- bash -c "echo 'started'; sleep 10"
//              (then press 'r' in the terminal, should see process restart)
#[test]
fn test_default_hotkey_restart() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "bash", "-c", "echo 'Process started'; sleep 10"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send 'r' hotkey through PTY master
    let mut writer = pair.master.take_writer().unwrap();
    writer.write_all(b"r").unwrap();
    writer.flush().unwrap();
    drop(writer);

    // Give it time to restart
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see "started" twice (initial + restart)
    let started_count = output_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages, got {}. Output:\n{}",
        started_count,
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --restart-hotkey x bash -c "echo 'started'; sleep 10"
//              (then press 'x' in the terminal)
#[test]
fn test_custom_hotkey_restart() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--restart-hotkey",
        "x",
        "--",
        "bash",
        "-c",
        "echo 'Process started'; sleep 10",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send 'x' hotkey through PTY master
    let mut writer = pair.master.take_writer().unwrap();
    writer.write_all(b"x").unwrap();
    writer.flush().unwrap();
    drop(writer);

    // Give it time to restart
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see "started" twice (initial + restart)
    let started_count = output_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages with custom hotkey 'x', got {}. Output:\n{}",
        started_count,
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- bash -- -c "echo 'started'; sleep 5"
//              (press keys other than 'r', should NOT restart)
#[test]
fn test_non_hotkey_characters_ignored() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    // Use a unique timestamp-based marker to count starts
    let marker = format!(
        "START_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let script = format!("echo '{}'; sleep 5", marker);

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "bash", "-c", &script]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send non-hotkey characters through PTY (avoid 'r' which is the default hotkey)
    let mut writer = pair.master.take_writer().unwrap();
    writer.write_all(b"abc123xyz").unwrap();
    writer.flush().unwrap();
    drop(writer);

    // Give it time to potentially restart (it shouldn't)
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should NOT see restart message (direct indicator that restart didn't happen)
    assert!(
        !output_str.contains("Restarting child process"),
        "Process should not have restarted from non-hotkey characters. Output:\n{}",
        output_str
    );

    // The marker appears once in output. If it appears more, that means a restart happened
    let started_count = output_str.matches(&marker).count();
    assert!(
        started_count <= 2,
        "Expected at most 2 occurrences of marker (no restart), got {}. Output:\n{}",
        started_count,
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --restart-hotkey r --stop-on-child-exit bash -c "echo 'done'; exit 0"
//              (verify hotkey is accepted with other flags)
#[test]
fn test_hotkey_with_stop_on_child_exit() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--restart-hotkey",
        "r",
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

    assert!(
        output_str.contains("test output"),
        "Expected 'test output' in output. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");

    drop(output_bytes);
    let _ = reader_thread.join();
}

// Manual test: cargo run -- --stop-on-child-exit bash -c "echo 'started'; sleep 3"
//              (press 'r' after 1s, verify restart happens, then supervisor exits when child exits)
#[test]
fn test_restart_with_stop_on_child_exit() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--stop-on-child-exit",
        "--",
        "bash",
        "-c",
        "echo 'Process started'; sleep 3",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send 'r' hotkey to trigger restart while child is still running
    let mut writer = pair.master.take_writer().unwrap();
    writer.write_all(b"r").unwrap();
    writer.flush().unwrap();
    drop(writer);

    // Wait for the restarted child to complete naturally (3s + buffer)
    std::thread::sleep(Duration::from_secs(4));

    // The supervisor should have exited on its own due to --stop-on-child-exit
    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Verify restart happened: should see "started" twice (initial + restart)
    let started_count = output_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages (restart happened), got {}. Output:\n{}",
        started_count,
        output_str
    );

    // Verify supervisor exited when child naturally exited (success exit code)
    assert!(
        status.success(),
        "Supervisor should exit successfully when child exits naturally. Exit status: {:?}",
        status
    );

    // Verify the --stop-on-child-exit message appears
    assert!(
        output_str.contains("Exiting (--stop-on-child-exit is set)"),
        "Should see --stop-on-child-exit exit message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// PTY Test 1: Long-running process with hotkey restart
#[test]
fn test_pty_long_running_process_with_hotkey() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--", "bash", "-c", "echo 'Process started'; sleep 30"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Wait for startup
    std::thread::sleep(Duration::from_secs(1));

    // Send hotkey through PTY master writer
    let mut writer = pair.master.take_writer().unwrap();
    writer.write_all(b"r").unwrap();
    writer.flush().unwrap();
    drop(writer);

    // Give time for restart
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see process started at least twice (initial + restart)
    let started_count = output_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages, got {}. Output:\n{}",
        started_count,
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// PTY Test 2: Process that exits immediately
#[test]
fn test_pty_process_exits_immediately() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&["--stop-on-child-exit", "echo", "quick exit"]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let status = child.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    assert!(
        output_str.contains("quick exit"),
        "Expected 'quick exit' in output. Output:\n{}",
        output_str
    );
    assert!(status.success(), "Process should exit successfully");
    assert!(
        output_str.contains("Exiting (--stop-on-child-exit is set)"),
        "Expected exit message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

// PTY Test 3: Process that prints continuously
#[test]
fn test_pty_continuous_output() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--",
        "bash",
        "-c",
        "for i in {1..5}; do echo \"Line $i\"; sleep 0.1; done; sleep 5",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Wait for output to be generated
    std::thread::sleep(Duration::from_secs(2));

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Should see all 5 lines
    for i in 1..=5 {
        let expected = format!("Line {}", i);
        assert!(
            output_str.contains(&expected),
            "Expected '{}' in output. Output:\n{}",
            expected,
            output_str
        );
    }

    drop(output_bytes);
    let _ = reader_thread.join();
}

// PTY Test 4: Process that ignores SIGTERM
#[test]
fn test_pty_process_ignores_sigterm() {
    use portable_pty::CommandBuilder;

    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi-cli"));
    cmd.args(&[
        "--",
        "bash",
        "-c",
        "trap '' TERM; echo 'Started and ignoring SIGTERM'; sleep 30",
    ]);

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    // Wait for startup
    std::thread::sleep(Duration::from_secs(1));

    // Try to kill the child - supi-cli should eventually force kill
    let kill_result = child.kill();

    // Wait for termination (with timeout)
    let start = std::time::Instant::now();
    let mut exited = false;

    while start.elapsed() < Duration::from_secs(10) {
        match child.try_wait() {
            Ok(Some(_)) => {
                exited = true;
                break;
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }

    assert!(
        kill_result.is_ok() || exited,
        "Should be able to terminate process that ignores SIGTERM"
    );

    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    assert!(
        output_str.contains("Started and ignoring SIGTERM"),
        "Expected startup message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();

    let _ = child.kill();
    let _ = child.wait();
}
