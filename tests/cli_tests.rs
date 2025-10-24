use assert_cmd::Command;
use predicates::prelude::*;

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

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_missing_command_fails() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_simple_echo() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("echo")
        .arg("hello world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_stop_on_child_exit_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("echo")
        .arg("test message")
        .assert()
        .success()
        .stdout(predicate::str::contains("test message"))
        .stdout(predicate::str::contains("Exiting"));
}

#[test]
fn test_nonexistent_command() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("this_command_does_not_exist_xyz123")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to spawn"));
}

#[test]
fn test_stdout_forwarding() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("bash")
        .arg("--")
        .arg("-c")
        .arg("echo line1 && echo line2 && echo line3")
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"))
        .stdout(predicate::str::contains("line2"))
        .stdout(predicate::str::contains("line3"));
}

#[test]
fn test_stderr_forwarding() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("bash")
        .arg("--")
        .arg("-c")
        .arg("echo 'error message' >&2")
        .assert()
        .success()
        .stderr(predicate::str::contains("error message"));
}
