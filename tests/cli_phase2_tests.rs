// Phase 2: Signal Handling
// Tests for termination signals (SIGINT, SIGTERM) and restart signal (SIGUSR1)

mod cli_test_utils;

use cli_test_utils::create_pty_with_reader;
use portable_pty::CommandBuilder;
use std::time::Duration;

// Manual test: cargo run -- sleep 30
//              (note the PID shown, then in another terminal: kill -TERM <pid>)
#[test]
fn test_sigterm_graceful_shutdown() {
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
