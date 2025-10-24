// Phase 3: Interactive Hotkey
// Tests for hotkey-based restart functionality

mod test_utils;

use portable_pty::CommandBuilder;
use std::io::Write;
use std::time::Duration;
use test_utils::create_pty_with_reader;

// Manual test: cargo run -- bash -c "echo 'started'; sleep 10"
//              (then press 'r' in the terminal, should see process restart)
#[test]
fn test_default_hotkey_restart() {
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

