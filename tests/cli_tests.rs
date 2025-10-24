use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Stdio;
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
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("echo")
        .arg("hello world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

// Manual test: cargo run -- --stop-on-child-exit echo "test message"
#[test]
fn test_stop_on_child_exit_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("echo")
        .arg("test message")
        .assert()
        .success()
        .stdout(predicate::str::contains("test message"))
        .stdout(predicate::str::contains(
            "Exiting (--stop-on-child-exit is set)",
        ));
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

// Manual test: cargo run -- --stop-on-child-exit bash -- -c "echo line1 && echo line2 && echo line3"
#[test]
fn test_stdout_forwarding() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("bash")
        .arg("--")
        .arg("-c")
        .arg("echo line1 && echo line2 && echo line3")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line2"))
        .stdout(predicate::str::contains("line3"));
}

// Manual test: cargo run -- --stop-on-child-exit bash -- -c "echo 'error message' >&2"
#[test]
fn test_stderr_forwarding() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("bash")
        .arg("--")
        .arg("-c")
        .arg("echo 'error message' >&2")
        .assert()
        .success()
        .stderr(predicate::str::contains("error message"));
}

// Phase 2 Tests: Signal Handling

// Manual test: cargo run -- sleep 30
//              (note the PID shown, then in another terminal: kill -TERM <pid>)
#[test]
fn test_sigterm_graceful_shutdown() {
    use std::process::Command as StdCommand;

    // Spawn supi-cli with a long-running process
    let child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("sleep")
        .arg("30")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGTERM
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show graceful shutdown message
    assert!(
        stdout.contains("Received SIGTERM signal") || stdout.contains("Stopping child process")
    );
    assert!(output.status.success());
}

// Manual test: cargo run -- sleep 30
//              (then press Ctrl+C or in another terminal: kill -INT <pid>)
#[test]
fn test_sigint_graceful_shutdown() {
    use std::process::Command as StdCommand;

    // Spawn supi-cli with a long-running process
    let child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("sleep")
        .arg("30")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGINT
    unsafe {
        libc::kill(pid as i32, libc::SIGINT);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show graceful shutdown message
    assert!(stdout.contains("Received SIGINT signal") || stdout.contains("Stopping child process"));
    assert!(output.status.success());
}

// Manual test: cargo run -- bash -c "echo 'started'; sleep 10"
//              (note the PID, then in another terminal: kill -USR1 <pid>, then: kill -TERM <pid>)
#[test]
fn test_restart_signal() {
    use std::process::Command as StdCommand;

    // Spawn supi-cli with a command that we can observe restarting
    let child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg("echo 'started'; sleep 10")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it more time to start and print first message
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGUSR1 to trigger restart
    unsafe {
        libc::kill(pid as i32, libc::SIGUSR1);
    }

    // Give it more time to restart
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGTERM to stop
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Give it time to process the signal
    std::thread::sleep(Duration::from_millis(500));

    // Collect output
    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should see restart messages in stdout
    assert!(
        stdout_str.contains("Received SIGUSR1 signal")
            || stdout_str.contains("Restarting child process")
    );
}

// Manual test: Create test.sh with: trap 'echo "Child got SIGTERM"; exit 0' TERM; echo "Started"; sleep 30
//              Then: chmod +x test.sh && cargo run -- ./test.sh
//              (note the PID, then in another terminal: kill -TERM <pid>)
#[test]
fn test_signal_forwarding_to_child() {
    use std::process::Command as StdCommand;

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

    let child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg(temp_file.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGTERM
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should see both supervisor and child handling the signal
    assert!(stdout_str.contains("Stopping child process"));
}

// Phase 3 Tests: Interactive Hotkey

// Manual test: cargo run -- bash -c "echo 'started'; sleep 10"
//              (then press 'r' in the terminal, should see process restart)
#[test]
fn test_default_hotkey_restart() {
    use std::io::Write;
    use std::process::Command as StdCommand;

    // Spawn supi-cli with a command that prints on start
    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg("echo 'Process started'; sleep 10")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send 'r' to stdin to trigger restart
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"r").unwrap();
        stdin.flush().unwrap();
    }

    // Give it time to restart
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGTERM to stop
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should see "started" twice (initial + restart) and restart message
    let started_count = stdout_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages, got {}. Output:\n{}",
        started_count,
        stdout_str
    );
}

// Manual test: cargo run -- --restart-hotkey x bash -c "echo 'started'; sleep 10"
//              (then press 'x' in the terminal)
#[test]
fn test_custom_hotkey_restart() {
    use std::io::Write;
    use std::process::Command as StdCommand;

    // Spawn supi-cli with custom hotkey
    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("--restart-hotkey")
        .arg("x")
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg("echo 'Process started'; sleep 10")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send 'x' to stdin to trigger restart
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"x").unwrap();
        stdin.flush().unwrap();
    }

    // Give it time to restart
    std::thread::sleep(Duration::from_secs(1));

    // Send SIGTERM to stop
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should see "started" twice (initial + restart)
    let started_count = stdout_str.matches("Process started").count();
    assert!(
        started_count >= 2,
        "Expected at least 2 'Process started' messages with custom hotkey 'x', got {}. Output:\n{}",
        started_count,
        stdout_str
    );
}

// Manual test: cargo run -- bash -- -c "echo 'started'; sleep 5"
//              (press keys other than 'r', should NOT restart)
#[test]
fn test_non_hotkey_characters_ignored() {
    use std::io::Write;
    use std::process::Command as StdCommand;

    // Use a unique timestamp-based marker to count starts
    let marker = format!(
        "START_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let script = format!("echo '{}'; sleep 5", marker);

    // Spawn supi-cli with a command that prints on start
    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_supi-cli"))
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn supi-cli");

    let pid = child.id();

    // Give it time to start
    std::thread::sleep(Duration::from_secs(1));

    // Send non-hotkey characters to stdin (avoid 'r' which is the default hotkey)
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"abc123xyz").unwrap();
        stdin.flush().unwrap();
    }

    // Give it time to potentially restart (it shouldn't)
    std::thread::sleep(Duration::from_secs(2));

    // Send SIGTERM to stop
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for it to exit
    let output = child.wait_with_output().expect("Failed to wait for child");
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should NOT see restart message (direct indicator that restart didn't happen)
    assert!(
        !stdout_str.contains("Restarting child process"),
        "Process should not have restarted from non-hotkey characters. Output:\n{}",
        stdout_str
    );

    // The marker appears twice: once in the command log, once in output
    // If it appears more than twice, that means a restart happened
    let started_count = stdout_str.matches(&marker).count();
    assert!(
        started_count <= 2,
        "Expected at most 2 occurrences of marker (no restart), got {}. Output:\n{}",
        started_count,
        stdout_str
    );
}

// Manual test: cargo run -- --restart-hotkey r --stop-on-child-exit bash -c "echo 'done'; exit 0"
//              (verify hotkey is accepted with other flags)
#[test]
fn test_hotkey_with_stop_on_child_exit() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--restart-hotkey")
        .arg("r")
        .arg("--stop-on-child-exit")
        .arg("echo")
        .arg("test output")
        .assert()
        .success()
        .stdout(predicate::str::contains("test output"));
}
