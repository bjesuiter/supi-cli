# Supi CLI - Technical Implementation Reference

## Overview

A lightweight process supervisor in Rust that manages child processes with
restart capabilities via signals and hotkeys.

**Current Status:** Version 1.0.0 - Core features complete, published to
crates.io\
**Implementation History:** See [history/](../history/) for detailed development
notes\
**User Documentation:** See [README.md](../README.md) for usage and roadmap

## Project Status

✅ **Phases 1-5 Complete:** Core functionality (process spawning, signals,
hotkeys, testing, advanced features)\
✅ **Phase 6 Complete:** Published to crates.io\

⏳ **Phase 7 In Progress:** Polish, distribution, CI/CD\
🔮 **Phase 8-9 Future:** Vim-style interactive mode and optional TUI (see
[README.md](../README.md))

## Module Architecture

### Core Components

**1. CLI Argument Parsing (`src/cli.rs`)**

- Library: `clap` v4 with derive macros
- Parses command and args, handles flags
- Key flags: `--stop-on-child-exit`, `--restart-signal`, `--restart-hotkey`,
  `--restart-debounce-ms`, `--silent`, `--log-color`, `--info-color`

**2. Process Management (`src/process.rs`)**

- Library: `tokio::process` with `nix` for Unix process control
- Spawns child in new process group for proper signal handling
- Captures and forwards stdout/stderr in real-time using async BufReader
- Graceful shutdown with SIGTERM → 5s timeout → SIGKILL
- Restart capability with configurable debouncing

**3. Signal Handling (`src/signals.rs`)**

- Libraries: `signal-hook` and `signal-hook-tokio` for async Unix signal
  handling
- Handles: SIGTERM, SIGINT, SIGQUIT (graceful shutdown)
- Configurable restart signal: SIGUSR1 (default), SIGUSR2, or SIGHUP
- Sends signals to child process group with timeout-based force kill

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

**Integration Tests:** 34 PTY-based tests using `portable-pty` for realistic
terminal behavior

- Organized by phase: `cli_phase1_tests.rs` through `cli_phase5_tests.rs`
- Bug-specific files: `cli_bugfix_process_group_cleanup.rs`
- Coverage: process spawning, signals, hotkeys, output forwarding, debouncing,
  graceful shutdown

**Unit Tests:** CLI parsing, signal name validation, hotkey validation

**Manual Testing Scenarios:** Long-running processes, immediate exits,
continuous output, SIGTERM-resistant processes, rapid restarts

### Dependencies

See [Cargo.toml](../Cargo.toml) for the complete dependency list with versions.

**Key Dependencies:**

- **CLI & Async:** `clap` (argument parsing), `tokio` (async runtime),
  `tokio-stream`
- **Terminal:** `crossterm` (raw mode, colors, events), `nix` (Unix signals)
- **Signals:** `signal-hook`, `signal-hook-tokio` (async signal handling)
- **Testing:** `assert_cmd`, `predicates`, `portable-pty` (PTY-based integration
  tests)

### Platform Support

**Primary Targets:**

- **macOS:** Apple Silicon (aarch64-apple-darwin)
- **Linux:** glibc (x86_64-unknown-linux-gnu) and musl
  (x86_64-unknown-linux-musl)

**Limitations:**

- Unix-only (relies on Unix signals: SIGUSR1, SIGUSR2, SIGHUP, SIGTERM, SIGINT)
- macOS Intel support possible but not officially tested/built

## Key Technical Challenges

### Challenge 1: Concurrent Event Handling ✅ SOLVED

**Problem**: Multiple async events (signals, input, process I/O) need
coordination\
**Solution**: Use `tokio::select!` to multiplex events in main loop

### Challenge 2: Clean Process Termination ✅ SOLVED

**Problem**: Ensure child process and all descendants are cleaned up properly\
**Solution**: Process groups + SIGTERM to group → 5s timeout → SIGKILL to
group + RAII cleanup pattern

### Challenge 3: Raw Terminal Mode Cleanup ✅ SOLVED

**Problem**: Terminal remains in raw mode if app crashes\
**Solution**: Crossterm's RAII cleanup + custom panic handler restoration

### Challenge 4: Output Forwarding Without Delay ✅ SOLVED

**Problem**: Buffering delays output visibility\
**Solution**: Line-based async reading with `tokio::io::BufReader`, no
additional buffering

### Challenge 5: Cross-Platform Signal Handling ✅ DOCUMENTED

**Problem**: Unix signals don't exist on Windows\
**Solution**: Unix-only implementation with conditional compilation; Windows
support possible in future via named events

### Challenge 6: Vim-Style Mode Switching (🔮 Phase 8 - Future)

**Problem**: Toggle between hotkey mode and stdin-forwarding mode\
**Solution Plan**: Mode state (Normal/Insert) + dynamic raw mode toggling +
stdin channel to child + ESC detection + visual mode indicator

### Challenge 7: Restart Debouncing ✅ SOLVED

**Problem**: Prevent accidental rapid restarts from hotkey mashing or multiple
signals\
**Solution**: Track last restart with `tokio::time::Instant` + configurable
`--restart-debounce-ms` (default 1000ms, set to 0 to disable)

### Challenge 8: TUI Mode Integration (🔮 Phase 9 - Future)

**Problem**: Render TUI while forwarding real-time child output\
**Solution Plan**: Buffered scrollable output widget + separate render loop +
terminal resize handling + balanced refresh rate

## Distribution

**Current:** Published to [crates.io](https://crates.io/crates/supi-cli) -
install with `cargo install supi-cli`

**Planned Targets for Pre-built Binaries:**

- `aarch64-apple-darwin` - Apple Silicon macOS
- `x86_64-unknown-linux-gnu` - Linux with glibc
- `x86_64-unknown-linux-musl` - Linux static binary (most portable)

**Build Commands:**

```bash
cargo build --release --target aarch64-apple-darwin      # macOS ARM64
cargo build --release --target x86_64-unknown-linux-gnu  # Linux GNU
cargo build --release --target x86_64-unknown-linux-musl # Linux MUSL
```

**Future CI/CD (Phase 7):**

- GitHub Actions with cross-compilation matrix
- Automated testing on macOS and Linux runners
- Release artifacts with GitHub Releases

## Implementation Timeline

**Completed Phases:**

- ✅ Phase 1-5: Core functionality (11-14 hours)
- ✅ Phase 6: Crates.io deployment (1-2 hours)

**In Progress:**

- ⏳ Phase 7: Polish & distribution (CI/CD, pre-built binaries)

**Future (Optional):**

- 🔮 Phase 8: Vim-style interactive mode with stdin forwarding (3-4 hours
  estimated)
- 🔮 Phase 9: Optional TUI mode with ratatui (3-5 hours estimated)

## Success Criteria

**Core Features (✅ Complete):**

- Spawns and supervises arbitrary processes
- Real-time stdout/stderr forwarding
- Unix signal handling (restart: SIGUSR1/SIGUSR2/SIGHUP, terminate:
  SIGINT/SIGTERM/SIGQUIT)
- Interactive hotkey support with configurable key
- Graceful shutdown with process group cleanup
- Configurable via CLI flags (colors, debouncing, silent mode)
- Works on Linux and macOS
- Comprehensive test suite (34 PTY-based integration tests)

**Distribution (✅ Phase 6 Complete, ⏳ Phase 7 In Progress):**

- ✅ Published to crates.io (version 1.0.0)
- ✅ Easy installation via `cargo install supi-cli`
- ✅ Comprehensive README with usage examples
- ⏳ CI/CD pipeline for automated testing and releases
- ⏳ Pre-built binaries for macOS and Linux

**Future Enhancements (🔮 Optional):**

- Vim-style interactive mode with stdin forwarding (Phase 8)
- Optional TUI mode for enhanced monitoring (Phase 9)
