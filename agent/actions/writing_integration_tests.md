# Writing Integration Tests for supi-cli

## Setup

### 1. Add Test Dependencies to Cargo.toml

```toml
[dev-dependencies]
assert_cmd = "2.0"      # Easy CLI testing
predicates = "3.0"      # Flexible assertions
tempfile = "3.8"        # Temporary files/dirs
```

### 2. Create tests/ Directory

Integration tests go in `tests/` at project root. Each file is a separate test
suite.

```
tests/
├── cli_tests.rs           # Basic CLI functionality
├── process_spawn_tests.rs # Process spawning
├── output_tests.rs        # Output forwarding
├── signal_tests.rs        # Signal handling
└── integration_helpers/   # Shared test utilities
    └── mod.rs
```

## Test Patterns for Process Supervisor

### Pattern 1: Basic CLI Tests

```rust
// tests/cli_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("process supervisor"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("supi-cli"));
}

#[test]
fn test_missing_command_fails() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
```

### Pattern 2: Process Spawning Tests

```rust
// tests/process_spawn_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

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
fn test_stop_on_child_exit() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("--stop-on-child-exit")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Exiting"));
}

#[test]
fn test_nonexistent_command() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("this_command_does_not_exist_12345")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to spawn"));
}
```

### Pattern 3: Output Forwarding Tests

```rust
// tests/output_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_stdout_forwarding() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("bash")
        .arg("--")  // Important: separates supi-cli args from command args
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
        .arg("-c")
        .arg("echo 'error message' >&2")
        .assert()
        .success()
        .stderr(predicate::str::contains("error message"));
}

#[test]
fn test_mixed_output() {
    let mut cmd = Command::cargo_bin("supi-cli").unwrap();
    cmd.arg("bash")
        .arg("-c")
        .arg("echo stdout1 && echo stderr1 >&2 && echo stdout2")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdout1"))
        .stdout(predicate::str::contains("stdout2"))
        .stderr(predicate::str::contains("stderr1"));
}
```

### Pattern 4: Signal Handling Tests (Advanced)

```rust
// tests/signal_tests.rs
use assert_cmd::Command;
use std::process::{Command as StdCommand, Stdio};
use std::time::Duration;
use std::thread;

#[test]
#[cfg(unix)]
fn test_sigterm_graceful_shutdown() {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    
    // Start supi in background with long-running process
    let mut child = StdCommand::new("cargo")
        .args(&["run", "--", "sleep", "30"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    
    // Give it time to start
    thread::sleep(Duration::from_millis(500));
    
    // Send SIGTERM
    let pid = Pid::from_raw(child.id() as i32);
    kill(pid, Signal::SIGTERM).unwrap();
    
    // Should exit gracefully within timeout
    let result = child.wait_timeout(Duration::from_secs(5)).unwrap();
    assert!(result.is_some(), "Process should exit on SIGTERM");
}

#[test]
#[cfg(unix)]
fn test_sigint_graceful_shutdown() {
    // Similar to above but with SIGINT (Ctrl+C)
    // ...
}
```

### Pattern 5: Helper Utilities

```rust
// tests/integration_helpers/mod.rs

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::thread;

/// Helper to start supi-cli in background
pub fn start_supervisor(args: &[&str]) -> Child {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "--"]);
    cmd.args(args);
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start supervisor")
}

/// Wait for output to contain string
pub fn wait_for_output(child: &mut Child, expected: &str, timeout: Duration) -> bool {
    use std::io::{BufRead, BufReader};
    
    let start = std::time::Instant::now();
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        if start.elapsed() > timeout {
            return false;
        }
        if let Ok(line) = line {
            if line.contains(expected) {
                return true;
            }
        }
    }
    false
}

/// Create temporary script file
pub fn create_test_script(content: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "#!/bin/bash").unwrap();
    writeln!(file, "{}", content).unwrap();
    
    // Make executable
    let metadata = file.as_file().metadata().unwrap();
    let mut perms = metadata.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(file.path(), perms).unwrap();
    
    file
}
```

## Test Organization Strategy

### Phase-Based Testing

Match tests to implementation phases:

```rust
// Phase 1: CLI & Skeleton
#[test] fn test_cli_help() { }
#[test] fn test_cli_version() { }
#[test] fn test_cli_arg_parsing() { }

// Phase 2: Basic Process Spawning
#[test] fn test_spawn_echo() { }
#[test] fn test_output_forwarding() { }
#[test] fn test_stop_on_exit_flag() { }

// Phase 3: Signal Handling
#[test] fn test_sigterm_shutdown() { }
#[test] fn test_sigint_shutdown() { }
#[test] fn test_restart_signal() { }

// Phase 4: Interactive Hotkey
#[test] fn test_hotkey_restart() { }

// Phase 5: Advanced Features
#[test] fn test_rapid_restarts() { }
#[test] fn test_custom_signals() { }
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test cli_tests

# Run specific test
cargo test test_help_flag

# With output
cargo test -- --nocapture

# Using bonnie
bx test                    # All tests
bx test cli_tests          # Specific file
bx test -- test_help_flag  # Specific test
```

## Best Practices

1. **Keep tests fast**: Use timeouts, avoid long sleeps
2. **Test isolation**: Each test should be independent
3. **Clear names**: `test_<feature>_<scenario>_<expected>`
4. **Use helpers**: Extract common patterns to helper functions
5. **Platform-specific**: Use `#[cfg(unix)]` for signal tests
6. **Timeouts**: Always set timeouts for long-running processes
7. **Cleanup**: Use RAII or Drop to ensure cleanup

## Limitations & Alternatives

### Cannot easily test with assert_cmd:

- Interactive TTY behavior (hotkeys)
- Long-running supervision loops
- Complex signal interactions

### For these, use:

- Manual testing scripts
- End-to-end tests with `expect` crate
- Subprocess-based integration tests with nix crate

## Next Steps

1. Start with Phase 1-2 tests (CLI, basic spawning)
2. Add Phase 3 tests after signal handling implemented
3. Consider separate E2E test suite for complex scenarios
4. Add CI workflow to run tests on push
