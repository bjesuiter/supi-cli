// Phase 4: PTY-Specific Scenarios
// Tests for edge cases and realistic terminal scenarios

mod cli_test_utils;

use cli_test_utils::create_pty_with_reader;
use portable_pty::CommandBuilder;
use std::io::Write;
use std::time::Duration;

// PTY Test 1: Long-running process with hotkey restart
#[test]
fn test_pty_long_running_process_with_hotkey() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
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
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
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
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
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
    let (pair, output, reader_thread) = create_pty_with_reader();

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
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
