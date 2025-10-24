# Supi CLI - Implementation Plan

## Overview

Building a lightweight process supervisor in Rust that manages child processes
with restart capabilities via signals and hotkeys.

## AI Agent Instructions

Let's implement the app! Let's do it step by step so that i can review it.
Answer concise, sacrifice grammar for brevity. I'll ask if i need more
explanation. Use @PROGRESS.md to figure out how much is done already and what to
do next. Use @actions/log_progress.md to log progress. Write integration tests
after each phase as defined in @actions/writing_integration_tests.md.

## Architecture Components

### 1. CLI Argument Parsing

- **Library**: `clap` v4 with derive macros
- **Features**: Parse command and args, handle flags
- **Arguments to implement**:
  - `--stop-on-child-exit`: Boolean flag
  - `--restart-signal <SIGNAL>`: Signal name (default: SIGUSR1)
  - `--restart-hotkey <KEY>`: Single character (default: 'r')
  - Positional args: Command and its arguments

### 2. Process Management

- **Library**: `tokio` with process feature
- **Responsibilities**:
  - Spawn child process with command + args
  - Capture and forward stdout/stderr
  - Track process state (running, stopped, exit code)
  - Gracefully terminate child on shutdown
  - Restart child process on demand

### 3. Signal Handling

- **Library**: `tokio::signal` for Unix signals
- **Signals to handle**:
  - User-configurable restart signal (default: SIGUSR1)
  - SIGTERM - graceful shutdown
  - SIGINT - graceful shutdown (Ctrl+C)
  - SIGQUIT - graceful shutdown
- **Behavior**:
  - Forward termination signals to child
  - Wait for child to exit gracefully
  - Force kill if child doesn't exit within timeout

### 4. Terminal Input (Hotkey Detection)

- **Library**: `crossterm` for cross-platform terminal manipulation
- **Features**:
  - Enable raw mode to capture single keystrokes
  - Non-blocking input reading
  - Detect restart hotkey
  - Clean terminal state on exit

### 5. Output Forwarding

- **Approach**: Async streams with tokio
- **Requirements**:
  - Forward stdout line-by-line in real-time
  - Forward stderr line-by-line in real-time
  - No buffering delays
  - Preserve output order as much as possible
  - Use `BufReader` with tokio's `AsyncBufReadExt`

### 6. Main Event Loop

- **Structure**: Tokio async runtime with select! macro
- **Events to handle**:
  - Child process stdout data
  - Child process stderr data
  - Child process exit
  - Unix signals (restart, terminate)
  - Terminal input (hotkey press)
  - Graceful shutdown coordination

## Technical Implementation Details

### Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.40", features = ["full"] }
tokio-util = { version = "0.7", features = ["io"] }
crossterm = "0.28"
anyhow = "1.0"
signal-hook = "0.3"
signal-hook-tokio = { version = "0.3", features = ["futures-v0_3"] }
```

### Module Structure

```
src/
├── main.rs           - Entry point, CLI setup, main loop
├── cli.rs            - Clap CLI argument definitions
├── process.rs        - Process spawning and management
├── signals.rs        - Signal handling setup
├── hotkey.rs         - Terminal input and hotkey detection
└── supervisor.rs     - Main supervisor coordination logic
```

## Implementation Phases

### Phase 1: Basic Process Spawning

- [x] Set up Clap CLI argument parsing
- [x] Parse command and arguments
- [x] Spawn child process using tokio::process::Command
- [x] Forward stdout/stderr to parent's stdout/stderr
- [x] Wait for process to exit
- [x] Basic error handling

### Phase 2: Signal Handling

- [x] Set up signal handlers for SIGINT, SIGTERM, SIGQUIT
- [x] Implement graceful shutdown (send SIGTERM to child, wait, force kill if
      needed)
- [x] Add configurable restart signal (default SIGUSR1)
- [x] Implement restart logic (terminate child, respawn)
- [x] Test signal handling

### Phase 3: Interactive Hotkey

- [x] Set up crossterm raw mode
- [x] Create async task for reading terminal input
- [x] Detect restart hotkey press
- [x] Trigger restart on hotkey
- [x] Handle terminal cleanup on exit
- [x] Make hotkey configurable

### Phase 4: Advanced Features

- [x] Implement --stop-on-child-exit flag
- [x] Handle child process exit scenarios
- [ ] Add restart debouncing (prevent rapid restarts)
- [ ] Improve error messages and logging
- [ ] Add process restart counter/statistics

### Phase 5: Polish & Distribution

- [ ] Comprehensive error handling
- [ ] Add informative status messages
- [ ] Test on Linux and macOS
- [ ] Set up CI/CD for cross-compilation
- [ ] Build release binaries for target platforms
- [ ] Documentation improvements
- [ ] Add examples directory

### Phase 6: Vim-Style Interactive Mode (Future)

- [ ] Add terminal mode state machine (Normal/Insert modes)
- [ ] Normal mode: Raw mode for hotkeys (current behavior)
  - Hotkeys active (restart, quit, etc.)
  - No stdin forwarding to child
- [ ] Insert mode: Pass-through mode for stdin
  - Press 'i' to enter insert mode
  - Forward stdin to child process
  - Press ESC to return to normal mode
  - Visual indicator showing current mode
- [ ] Add --interactive flag to start in insert mode
- [ ] Display mode indicator (e.g., "-- INSERT --" or "-- NORMAL --")
- [ ] Smooth mode transitions without disrupting child process

## Key Technical Challenges

### Challenge 1: Concurrent Event Handling

**Problem**: Multiple async events (signals, input, process I/O) need
coordination **Solution**: Use `tokio::select!` to multiplex events in main loop

### Challenge 2: Clean Process Termination

**Problem**: Ensure child process is always cleaned up properly **Solution**:

- Use RAII pattern with Drop trait
- Implement timeout-based forced termination
- Handle zombie processes

### Challenge 3: Raw Terminal Mode Cleanup

**Problem**: If app crashes, terminal may remain in raw mode **Solution**:

- Use crossterm's automatic cleanup
- Implement custom panic handler to restore terminal
- Test with various exit scenarios

### Challenge 4: Output Forwarding Without Delay

**Problem**: Buffering can delay output visibility **Solution**:

- Use line-based async reading with BufReader
- Don't add additional buffering
- Use `tokio::io::copy` or manual forwarding loop

### Challenge 5: Cross-Platform Signal Handling

**Problem**: Signals work differently on Unix vs Windows **Solution**:

- Use conditional compilation for Unix-specific signals
- Document Unix-only requirement
- Consider future Windows support with named events

### Challenge 6: Vim-Style Mode Switching (Phase 6)

**Problem**: Toggle between raw mode (hotkeys) and cooked mode (stdin
forwarding) **Solution**:

- Maintain mode state (Normal/Insert)
- Disable terminal raw mode when entering insert mode
- Re-enable raw mode when returning to normal mode
- Use channel to communicate stdin data to child process
- Display visual indicator of current mode
- Handle ESC key detection in insert mode to return to normal
- Ensure smooth transitions without disrupting child output

## Testing Strategy

### Unit Tests

- CLI argument parsing edge cases
- Signal name validation
- Hotkey character validation

### Integration Tests

- Spawn and terminate simple processes
- Test restart functionality
- Signal handling with mock processes
- Output forwarding correctness

### Manual Testing Scenarios

1. Long-running process (sleep infinity)
2. Process that exits immediately
3. Process that prints continuously
4. Process that ignores SIGTERM
5. Rapid restart requests
6. Signal handling while process is restarting

## Distribution Targets

Building static binaries for:

- `aarch64-apple-darwin` (Apple Silicon macOS)
- `x86_64-unknown-linux-gnu` (Linux with glibc)
- `x86_64-unknown-linux-musl` (Linux static binary)

### Build Process

```bash
# macOS ARM64
cargo build --release --target aarch64-apple-darwin

# Linux GNU
cargo build --release --target x86_64-unknown-linux-gnu

# Linux MUSL (static)
cargo build --release --target x86_64-unknown-linux-musl
```

### GitHub Actions CI/CD

- Set up cross-compilation matrix
- Build on appropriate runners (macos-latest, ubuntu-latest)
- Create release artifacts with version tags
- Run tests on each platform

## Error Handling Strategy

- Use `anyhow` for application-level errors with context
- Use `Result<T>` throughout for proper error propagation
- Provide helpful error messages for common issues:
  - Command not found
  - Permission denied
  - Invalid signal names
  - Terminal access issues

## Future Enhancements (Out of Scope)

- Configuration file support
- Multiple process supervision
- Process groups and dependencies
- Log file rotation
- Web UI for status monitoring
- Automatic restart on file changes (file watching)
- Windows support

## Implementation Timeline

- **Phase 1**: 2-3 hours
- **Phase 2**: 2-3 hours
- **Phase 3**: 2-3 hours
- **Phase 4**: 1-2 hours
- **Phase 5**: 2-3 hours
- **Phase 6**: 3-4 hours (future enhancement)
- **Total**: ~10-15 hours for core implementation, ~13-19 hours with interactive
  mode

## Success Criteria

✅ Can spawn and supervise arbitrary processes ✅ Forwards stdout/stderr in
real-time ✅ Responds to Unix signals (restart and terminate) ✅ Interactive
hotkey works reliably ✅ Graceful shutdown with child cleanup ✅ Configurable
via CLI flags ✅ Works on Linux and macOS ✅ Clean, maintainable code with good
error handling ✅ Comprehensive README with examples ✅ Release binaries for all
target platforms
