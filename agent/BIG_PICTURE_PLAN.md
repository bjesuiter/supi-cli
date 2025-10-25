# Supi CLI - Technical Implementation Reference

## Overview

Building a lightweight process supervisor in Rust that manages child processes
with restart capabilities via signals and hotkeys.

**Current Status:** Phases 1-5 complete (see
[history/2025-10-25_INITIAL_IMPLEMENTATION.md](../history/2025-10-25_INITIAL_IMPLEMENTATION.md))\
**This Document:** Technical reference for architecture and challenges

**For user-facing roadmap:** See "Future Considerations and Improvements" in
[README.md](../README.md)

## Completed Phases

✅ **Phase 1:** Basic Process Spawning\
✅ **Phase 2:** Signal Handling\
✅ **Phase 3:** Interactive Hotkey\
✅ **Phase 4:** PTY-Based Testing\
✅ **Phase 5:** Advanced Features (colored logging, --silent, debouncing)

See
[history/2025-10-25_INITIAL_IMPLEMENTATION.md](../history/2025-10-25_INITIAL_IMPLEMENTATION.md)
for detailed implementation notes.

## Future Work Overview

See [README.md](../README.md) "Future Considerations and Improvements" section
for user-facing roadmap.

**Technical phases:**

- **Phase 6:** Crates.io deployment and packaging
- **Phase 7:** Polish, distribution, CI/CD
- **Phase 8:** Vim-style interactive mode (stdin forwarding)
- **Phase 9:** Optional TUI mode with ratatui

## Module Architecture

### Core Components

**1. CLI Argument Parsing (`src/cli.rs`)**

- Library: `clap` v4 with derive macros
- Parses command and args, handles flags
- Key flags: `--stop-on-child-exit`, `--restart-signal`, `--restart-hotkey`,
  `--restart-debounce-ms`, `--silent`, `--log-color`, `--info-color`

**2. Process Management (`src/process.rs`)**

- Library: `tokio` with process feature
- Spawns child process, captures/forwards stdout/stderr
- Tracks process state, handles graceful termination
- Restart capability with configurable debouncing

**3. Signal Handling (`src/signals.rs`)**

- Library: `tokio::signal` for Unix signals
- Handles: SIGTERM, SIGINT, SIGQUIT (graceful shutdown)
- Configurable restart signal (default: SIGUSR1)
- Forwards signals to child, timeout-based force kill

**4. Terminal Input (`src/hotkey.rs`)**

- Library: `crossterm` for terminal manipulation
- Raw mode for single keystroke capture
- Non-blocking input with RAII cleanup
- Configurable restart hotkey (default: 'r')

**5. Output Management (`src/output.rs`)**

- Stateful `Output` struct for colored, suppressible logging
- Separate colors for logs vs info messages
- Silent mode (suppress supervisor logs, keep child output)
- Thread-safe with internal mutex

**6. Supervisor (`src/supervisor.rs`)**

- Main event loop using `tokio::select!`
- Coordinates signals, hotkeys, process I/O
- Restart debouncing logic
- Graceful shutdown coordination

### Module Structure

```
src/
├── main.rs           - Entry point, CLI setup, main loop
├── cli.rs            - Clap CLI argument definitions
├── process.rs        - Process spawning and management
├── signals.rs        - Signal handling setup
├── hotkey.rs         - Terminal input and hotkey detection
├── output.rs         - Colored, stateful output management
└── supervisor.rs     - Main supervisor coordination logic
```

### Error Handling Strategy

- Use `anyhow` for application-level errors with context
- Use `Result<T>` throughout for proper error propagation
- Provide helpful error messages for:
  - Command not found
  - Permission denied
  - Invalid signal names
  - Terminal access issues

### Testing Strategy

**Unit Tests:**

- CLI argument parsing edge cases
- Signal name validation
- Hotkey character validation

**Integration Tests (via PTY):**

- All 34 tests use `portable-pty` for realistic terminal behavior
- Tests organized by phase in separate files (`tests/cli_phase1_tests.rs`
  through `cli_phase5_tests.rs`)
- Bug-specific test files with descriptive names (e.g.,
  `cli_bugfix_process_group_cleanup.rs`)
- Clean test output without raw mode artifacts
- Coverage: process spawning, signals, hotkeys, output forwarding, debouncing

**Manual Testing:**

- Long-running processes
- Processes that exit immediately
- Continuous output
- Stubborn processes ignoring SIGTERM
- Rapid restart requests (debouncing)

### Dependencies

See [../Cargo.toml](../Cargo.toml) for full dependency list.

**Production Dependencies:**

- `clap` v4 (with derive) - CLI argument parsing
- `tokio` v1.40 (full features) - Async runtime and process management
- `tokio-util` v0.7 (io features) - Additional tokio utilities
- `crossterm` v0.28 - Terminal manipulation for hotkey and colored output
- `signal-hook` v0.3 - Signal handling foundation
- `signal-hook-tokio` v0.3 (futures-v0_3) - Async Unix signal handling
- `anyhow` v1.0 - Application-level error handling with context

**Dev Dependencies:**

- `assert_cmd` - Integration test command execution
- `predicates` - Assertions for test output
- `portable-pty` v0.8 - PTY emulation for realistic terminal testing

### Platform Support

**Target Platforms:**

- macOS (Apple Silicon, no Intel support)
- Linux (glibc & musl)

**Limitation:** Unix-only (signals not available on Windows)

## Key Technical Challenges

### Challenge 1: Concurrent Event Handling ✅ SOLVED

**Problem**: Multiple async events (signals, input, process I/O) need
coordination\
**Solution**: Use `tokio::select!` to multiplex events in main loop

### Challenge 2: Clean Process Termination ✅ SOLVED

**Problem**: Ensure child process is always cleaned up properly\
**Solution**:

- Use RAII pattern with Drop trait
- Implement timeout-based forced termination
- Handle zombie processes

### Challenge 3: Raw Terminal Mode Cleanup ✅ SOLVED

**Problem**: If app crashes, terminal may remain in raw mode\
**Solution**:

- Use crossterm's automatic cleanup
- Implement custom panic handler to restore terminal
- Test with various exit scenarios

### Challenge 4: Output Forwarding Without Delay ✅ SOLVED

**Problem**: Buffering can delay output visibility\
**Solution**:

- Use line-based async reading with BufReader
- Don't add additional buffering
- Use `tokio::io::copy` or manual forwarding loop

### Challenge 5: Cross-Platform Signal Handling ✅ DOCUMENTED

**Problem**: Signals work differently on Unix vs Windows\
**Solution**:

- Use conditional compilation for Unix-specific signals
- Document Unix-only requirement
- Consider future Windows support with named events

### Challenge 6: Vim-Style Mode Switching (Phase 8)

**Problem**: Toggle between raw mode (hotkeys) and cooked mode (stdin
forwarding)\
**Solution**:

- Maintain mode state (Normal/Insert)
- Disable terminal raw mode when entering insert mode
- Re-enable raw mode when returning to normal mode
- Use channel to communicate stdin data to child process
- Display visual indicator of current mode
- Handle ESC key detection in insert mode to return to normal
- Ensure smooth transitions without disrupting child output

### Challenge 7: Restart Debouncing ✅ SOLVED

**Problem**: Prevent accidental rapid restarts from user mashing hotkey or
sending multiple signals\
**Solution**:

- Track last restart timestamp using `tokio::time::Instant`
- Check elapsed time before allowing restart
- Configurable debounce window via `--restart-debounce-ms` (default: 1000ms)
- Setting to 0 disables debouncing for power users
- Apply debouncing to both hotkey and signal restart triggers
- Log informative messages when restart is debounced
- No blocking behavior - just silently ignore rapid requests

### Challenge 8: TUI Mode Integration (Phase 9)

**Problem**: Manage TUI rendering while forwarding child output in real-time\
**Solution**:

- Buffer child output in scrollable widget
- Separate render loop from output forwarding
- Handle terminal resize events gracefully
- Preserve raw terminal state across mode switches
- Maintain responsive UI with high-frequency child output
- Balance UI refresh rate with CPU usage
- Clean TUI teardown on exit or panic

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

### GitHub Actions CI/CD (Phase 7)

- Set up cross-compilation matrix
- Build on appropriate runners (macos-latest, ubuntu-latest)
- Create release artifacts with version tags
- Run tests on each platform

## Implementation Timeline

**Completed:**

- ✅ Phase 1: 2-3 hours
- ✅ Phase 2: 2-3 hours
- ✅ Phase 3: 2-3 hours
- ✅ Phase 4: 2-3 hours
- ✅ Phase 5: 1-2 hours

**Remaining:**

- Phase 6: 1-2 hours (crates.io deployment)
- Phase 7: 2-3 hours (polish & distribution)
- Phase 8: 3-4 hours (vim-style interactive mode - optional)
- Phase 9: 3-5 hours (TUI mode - optional)

**Total Completed**: ~11-14 hours\
**Remaining Core**: ~3-5 hours (Phases 6-7)\
**With Optional Enhancements**: ~9-14 hours (includes Phases 8-9)

## Success Criteria

**Core Features (Achieved):**

- ✅ Can spawn and supervise arbitrary processes
- ✅ Forwards stdout/stderr in real-time
- ✅ Responds to Unix signals (restart and terminate)
- ✅ Interactive hotkey works reliably
- ✅ Graceful shutdown with child cleanup
- ✅ Configurable via CLI flags
- ✅ Works on Linux and macOS
- ✅ Clean, maintainable code with good error handling
- ✅ Comprehensive test suite (34 tests)

**Distribution (Phase 6-7):**

- ✅ Published on crates.io
- [x] Easy installation with `cargo install` (docs updated)
- [x] Comprehensive README with examples
- [ ] Release binaries for all target platforms
- [ ] CI/CD pipeline

**Optional Enhancements (Phase 8-9):**

- [ ] Vim-style interactive mode with stdin forwarding
- [ ] Optional TUI mode for enhanced monitoring
