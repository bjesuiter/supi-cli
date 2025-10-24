# Agent Guide - Supi CLI Repository

This document helps AI agents navigate the repository structure and understand
where to find different types of information.

## Quick Navigation

- **User Documentation**: [README.md](README.md) - End-user facing documentation
- **Architecture & Implementation Plan**:
  [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md)
- **Current Progress**: [agent/PROGRESS.md](agent/PROGRESS.md)
- **Agent Guidelines**: [agent/actions/](agent/actions/)

## Repository Structure

### Source Code (`src/`)

The application is organized into focused modules:

- `main.rs` - Entry point, initializes supervisor and hotkey listener
- `cli.rs` - Command-line argument parsing (clap)
- `supervisor.rs` - Main event loop coordinating signals, hotkeys, and process
  lifecycle
- `process.rs` - Process spawning, restarting, and graceful shutdown
- `signals.rs` - Unix signal handling (SIGTERM, SIGINT, SIGUSR1, etc.)
- `hotkey.rs` - Terminal input capture with crossterm for restart hotkey

### Tests (`tests/`)

- `cli_tests.rs` - Integration tests for all implemented features
  - Run with: `cargo test` or `bx test`
  - Currently 13 tests covering Phases 1-3

### Agent Documentation (`agent/`)

#### Core Documents

- `BIG_PICTURE_PLAN.md` - Complete architecture, implementation phases,
  technical decisions
  - **Consult this for**: Architecture details, dependencies, module
    responsibilities, implementation phases
  - **Do not duplicate**: Information already documented here

- `PROGRESS.md` - Chronological log of what's been implemented
  - **Use this to**: Understand current state, what's done, what's next
  - **Update this**: After completing significant work (see logging guidelines
    below)

- `bonnie.toml-reference.md` - Task runner configuration reference

#### Action Guides (`agent/actions/`)

- `log_progress.md` - Guidelines for updating PROGRESS.md
- `writing_integration_tests.md` - Test writing strategy and patterns

## Development Workflow

### Task Runner

This project uses `bx` (custom fork of bonnie) for common tasks:

```bash
bx run         # Build and run in dev mode
bx build       # Build debug binary
bx release     # Build optimized release binary
bx test        # Run all tests
bx fmt         # Format code
bx lint        # Run clipper
```

See [bonnie.toml](bonnie.toml) for all available commands.

### Build Artifacts

Build outputs are in `target/`:

- `target/debug/` - Debug builds
- `target/release/` - Optimized release builds

## Implementation Status

**Current Phase**: Phase 4 (Advanced Features) - In Progress

**Completed**:

- ✅ Phase 1: Basic Process Spawning
- ✅ Phase 2: Signal Handling
- ✅ Phase 3: Interactive Hotkey

**Next Steps**: See [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) Phase
4 checklist

## Key Technical Details

For comprehensive technical information, see
[agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md), including:

- Architecture components and responsibilities
- Dependencies and their usage
- Implementation patterns (tokio::select!, async streams, etc.)
- Testing strategy
- Distribution targets
- Error handling approach

## Making Changes

### Before Implementing

1. Check [agent/PROGRESS.md](agent/PROGRESS.md) for current status
2. Review relevant section in
   [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md)
3. Look at existing tests in `tests/cli_tests.rs` for patterns

### After Implementing

1. Write integration tests (see
   [agent/actions/writing_integration_tests.md](agent/actions/writing_integration_tests.md))
2. Update [agent/PROGRESS.md](agent/PROGRESS.md) (see
   [agent/actions/log_progress.md](agent/actions/log_progress.md))
3. Run `bx test` to ensure all tests pass
4. Run `bx lint` and `bx fmt` for code quality

## Dependencies

See `Cargo.toml` for full dependency list. Key libraries:

- `clap` - CLI argument parsing
- `tokio` - Async runtime and process management
- `crossterm` - Terminal manipulation for hotkey
- `signal-hook-tokio` - Unix signal handling
- `anyhow` - Error handling

## Platform Support

**Target Platforms**:

- macOS (Apple Silicon without Intel)
- Linux (glibc & musl)

**Limitation**: Unix-only (signals not available on Windows)
