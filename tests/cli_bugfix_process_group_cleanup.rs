// Bug Fix Tests
// Tests for specific bug fixes to ensure they don't regress

mod cli_test_utils;

use cli_test_utils::create_pty_with_reader;
use portable_pty::CommandBuilder;
use std::time::Duration;

/// Test that process groups are properly killed when shutting down
///
/// Bug: When running commands like `bash -c "npm run build && npm run dev"`,
/// the child processes spawned by bash would not be killed when supi shut down.
/// Only the bash process itself was killed, leaving orphaned child processes.
///
/// Fix: Use process groups - spawn with `.process_group(0)` and send signals
/// to the entire process group (negative PID) instead of just the direct child.
///
/// This test verifies that when bash spawns child processes (sleep commands),
/// all processes in the tree are properly terminated on shutdown.
#[test]
fn test_process_group_cleanup_on_shutdown() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    // Create a script that spawns multiple child processes
    // This simulates the real-world case of `bash -c "npm run build && npm run dev"`
    let script = r#"#!/bin/bash
echo "Parent bash starting (PID: $$)"
# Spawn background sleep processes to simulate child commands
sleep 30 &
CHILD1=$!
echo "Spawned child 1 (PID: $CHILD1)"
sleep 30 &
CHILD2=$!
echo "Spawned child 2 (PID: $CHILD2)"
# Wait for all children
wait
echo "All children completed"
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

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.arg(temp_file.path().to_str().unwrap());

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let supi_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give processes time to start and spawn children
    std::thread::sleep(Duration::from_secs(2));

    // Find the bash process (child of supi)
    #[cfg(unix)]
    let bash_pid = {
        use std::process::Command;
        let output = Command::new("pgrep")
            .args(&["-P", &supi_pid.to_string()])
            .output()
            .expect("Failed to run pgrep");

        let pids_str = String::from_utf8_lossy(&output.stdout);
        pids_str
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .next()
    };

    // Find sleep processes (children of bash)
    #[cfg(unix)]
    let sleep_pids = {
        if let Some(bash_pid) = bash_pid {
            use std::process::Command;
            let output = Command::new("pgrep")
                .args(&["-P", &bash_pid.to_string()])
                .output()
                .expect("Failed to run pgrep");

            let pids_str = String::from_utf8_lossy(&output.stdout);
            pids_str
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect::<Vec<u32>>()
        } else {
            Vec::new()
        }
    };

    #[cfg(unix)]
    {
        // Verify that child processes were spawned
        assert!(bash_pid.is_some(), "Bash process should be running");
        assert!(
            !sleep_pids.is_empty(),
            "Sleep child processes should be running"
        );
    }

    // Send SIGTERM to supi to trigger shutdown
    #[cfg(unix)]
    unsafe {
        libc::kill(supi_pid as i32, libc::SIGTERM);
    }

    // Wait for shutdown to complete (with timeout)
    let _ = child.wait();
    std::thread::sleep(Duration::from_secs(2));

    // Verify all processes are gone
    #[cfg(unix)]
    {
        // Check if bash process is still running
        if let Some(bash_pid) = bash_pid {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-p", &bash_pid.to_string()])
                .output()
                .expect("Failed to run ps");

            let ps_output = String::from_utf8_lossy(&output.stdout);
            assert!(
                !ps_output.contains(&bash_pid.to_string()),
                "Bash process (PID: {}) should have been killed but is still running",
                bash_pid
            );
        }

        // Check if any sleep processes are still running
        for sleep_pid in sleep_pids {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-p", &sleep_pid.to_string()])
                .output()
                .expect("Failed to run ps");

            let ps_output = String::from_utf8_lossy(&output.stdout);
            assert!(
                !ps_output.contains(&sleep_pid.to_string()),
                "Sleep child process (PID: {}) should have been killed but is still running. \
                This indicates the process group cleanup is not working correctly.",
                sleep_pid
            );
        }
    }

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Verify shutdown happened gracefully
    assert!(
        output_str.contains("Stopping child process"),
        "Expected 'Stopping child process' message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}

/// Test that process groups are properly killed on restart (not just shutdown)
///
/// This ensures that when restarting, the old process tree is completely cleaned up
/// before starting the new one.
#[test]
fn test_process_group_cleanup_on_restart() {
    let (pair, output, reader_thread) = create_pty_with_reader();

    // Create a script that spawns multiple child processes
    // This simulates the real-world case of `bash -c "npm run build && npm run dev"`
    let script = r#"#!/bin/bash
echo "Parent bash starting (PID: $$)"
# Spawn background sleep processes to simulate child commands
sleep 30 &
CHILD1=$!
echo "Spawned child 1 (PID: $CHILD1)"
sleep 30 &
CHILD2=$!
echo "Spawned child 2 (PID: $CHILD2)"
# Wait for all children
wait
echo "All children completed"
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

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_supi"));
    cmd.arg(temp_file.path().to_str().unwrap());

    let mut child = pair.slave.spawn_command(cmd).unwrap();
    let supi_pid = child.process_id().unwrap();
    drop(pair.slave);

    // Give first process time to start and spawn children
    std::thread::sleep(Duration::from_secs(2));

    // Find the first bash process and its children
    #[cfg(unix)]
    let first_bash_pid = {
        use std::process::Command;
        let output = Command::new("pgrep")
            .args(&["-P", &supi_pid.to_string()])
            .output()
            .expect("Failed to run pgrep");

        let pids_str = String::from_utf8_lossy(&output.stdout);
        pids_str
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .next()
    };

    #[cfg(unix)]
    let first_sleep_pids = {
        if let Some(bash_pid) = first_bash_pid {
            use std::process::Command;
            let output = Command::new("pgrep")
                .args(&["-P", &bash_pid.to_string()])
                .output()
                .expect("Failed to run pgrep");

            let pids_str = String::from_utf8_lossy(&output.stdout);
            pids_str
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect::<Vec<u32>>()
        } else {
            Vec::new()
        }
    };

    #[cfg(unix)]
    {
        // Verify that child processes were spawned
        assert!(first_bash_pid.is_some(), "Bash process should be running");
        assert!(
            !first_sleep_pids.is_empty(),
            "Sleep child processes should be running"
        );
    }

    // Send SIGUSR1 to trigger restart
    #[cfg(unix)]
    unsafe {
        libc::kill(supi_pid as i32, libc::SIGUSR1);
    }

    // Wait for restart to complete
    std::thread::sleep(Duration::from_secs(2));

    // Verify the old process tree is completely gone
    #[cfg(unix)]
    {
        if let Some(bash_pid) = first_bash_pid {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-p", &bash_pid.to_string()])
                .output()
                .expect("Failed to run ps");

            let ps_output = String::from_utf8_lossy(&output.stdout);
            assert!(
                !ps_output.contains(&bash_pid.to_string()),
                "Old bash process (PID: {}) should have been killed on restart but is still running",
                bash_pid
            );
        }

        for sleep_pid in first_sleep_pids {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-p", &sleep_pid.to_string()])
                .output()
                .expect("Failed to run ps");

            let ps_output = String::from_utf8_lossy(&output.stdout);
            assert!(
                !ps_output.contains(&sleep_pid.to_string()),
                "Old sleep child process (PID: {}) should have been killed on restart but is still running. \
                This indicates the process group cleanup is not working correctly on restart.",
                sleep_pid
            );
        }
    }

    // Clean up - send SIGTERM to stop supi
    #[cfg(unix)]
    unsafe {
        libc::kill(supi_pid as i32, libc::SIGTERM);
    }

    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(500));

    let output_bytes = output.lock().unwrap();
    let output_str = String::from_utf8_lossy(&output_bytes);

    // Verify restart happened
    assert!(
        output_str.contains("Restarting child process")
            || output_str.contains("Received SIGUSR1 signal"),
        "Expected restart message. Output:\n{}",
        output_str
    );

    drop(output_bytes);
    let _ = reader_thread.join();
}
