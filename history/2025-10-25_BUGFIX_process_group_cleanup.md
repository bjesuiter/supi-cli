# Bug Fix: Process Group Cleanup

## Problem

When running commands with shell commands that spawn child processes, such as:

```bash
supi bash -- -c "npm run build:app && npm run dev"
```

The child processes spawned by bash were not being killed when the supervisor
shut down or restarted. Only the bash process itself was terminated, leaving
orphaned child processes (npm, node, etc.) running in the background.

## Root Cause

The supervisor was sending signals (SIGTERM/SIGKILL) only to the direct child
process (bash), not to the entire process tree. When bash was killed, its child
processes were left running as orphans.

## Solution

Implemented proper **process group management**:

1. **Create Process Group**: Modified `process.rs` to spawn child processes in
   their own process group using `.process_group(0)`
2. **Signal Process Group**: Modified the shutdown logic to send signals to the
   entire process group (using negative PID) instead of just the single process

### Code Changes

**File: `src/process.rs`**

1. Added `.process_group(0)` to the spawn command:

```rust
let mut child = Command::new(&self.command)
    .args(&self.args)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true)
    .process_group(0) // Create new process group
    .spawn()
    .context("Failed to spawn child process")?;
```

2. Updated signal sending to target the process group:

```rust
// Send SIGTERM to entire process group for graceful shutdown
// Negative PID targets the process group
let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
```

3. Added SIGKILL to process group on timeout:

```rust
Err(_) => {
    self.output
        .log("[supi] Child process didn't stop gracefully, forcing...");
    // Force kill the entire process group
    let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGKILL);
}
```

## Testing

Created comprehensive tests in `tests/cli_bugfix_tests.rs`:

1. **`test_process_group_cleanup_on_shutdown`**: Verifies that when bash spawns
   multiple sleep processes, all processes in the tree are properly terminated
   when SIGTERM is sent to supi
2. **`test_process_group_cleanup_on_restart`**: Verifies that the old process
   tree is completely cleaned up during restart before starting the new one

Both tests use the same sophisticated test script:

- Spawn a bash script that creates **two** child processes (simulating
  real-world scenarios like `npm run build && npm run dev`)
- Track PIDs of bash and all its children
- Trigger shutdown/restart
- Verify **all** processes are killed (not just the parent bash process)
- Assert with clear error messages if any process survives

## Results

✅ All 36 tests pass (34 existing + 2 new bug fix tests) ✅ Process trees are
now properly cleaned up on shutdown ✅ Process trees are properly cleaned up on
restart ✅ No orphaned processes left behind

## Technical Details

**Process Groups in Unix**: When a process is spawned with `.process_group(0)`,
it becomes the leader of a new process group with the same PGID as its PID. All
its child processes inherit this PGID.

**Signaling Process Groups**: Using a negative PID in `kill()` targets the
entire process group:

- `kill(pid, signal)` - kills single process
- `kill(-pid, signal)` - kills entire process group

This ensures that commands like `bash -c "npm run build && npm run dev"` have
all their descendant processes properly terminated.

## Verification

To manually verify the fix works:

```bash
# Terminal 1: Start supi with a command that spawns children
supi bash -- -c "sleep 30 & sleep 30 & wait"

# Terminal 2: Check processes
ps aux | grep sleep  # Should show sleep processes

# Terminal 3: Stop supi
pkill -TERM supi

# Terminal 2: Verify cleanup
ps aux | grep sleep  # Should show NO sleep processes
```

## CHANGELOG Entry

Added to CHANGELOG.md under [Unreleased]:

```markdown
### Fixed

- **Process tree cleanup**: Child processes spawned by the supervised command
  (e.g., `bash -c "npm run build && npm run dev"`) are now properly killed when
  the supervisor shuts down or restarts
  - Processes are now spawned in their own process group using
    `.process_group(0)`
  - Signals (SIGTERM/SIGKILL) are sent to the entire process group instead of
    just the direct child
  - This ensures that shell commands with child processes are completely
    terminated
```
