# Agent Guide - Supi CLI Repository

This document helps AI agents navigate the repository structure and understand
where to find different types of information.

## AI Agent Instructions

When working on this project:

- Implement step by step for review
- Answer concise, sacrifice grammar for brevity
- Check
  [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)
  for completed phases
- Look at [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) for future
  phases (6-9)
- Write integration tests after each phase per
  [agent/actions/writing_integration_tests.md](agent/actions/writing_integration_tests.md)

## Quick Navigation

- **User Documentation**: [README.md](README.md) - End-user facing documentation
- **Completed Work**:
  [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md) -
  Phases 1-5 implementation details
- **Future Work**: [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) -
  Phases 6-9 and architectural reference
- **Agent Guidelines**: [agent/actions/](agent/actions/) - Process documentation
- **Change History**: [history/](history/) - Detailed bug fix and feature
  documentation

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

Tests organized by phase for better maintainability:

- `cli_test_utils.rs` - Shared PTY setup helper function
- `cli_phase1_tests.rs` - CLI and basic process tests (9 tests)
- `cli_phase2_tests.rs` - Signal handling tests (4 tests)
- `cli_phase3_tests.rs` - Interactive hotkey tests (5 tests)
- `cli_phase4_tests.rs` - PTY-specific scenarios (4 tests)
- `cli_phase5_tests.rs` - Silent flag, color output, debounce tests (12 tests)
- `cli_bugfix_*.rs` - Specific test files for bug fixes with descriptive names
  - Example: `cli_bugfix_process_group_cleanup.rs`

**Run tests:** `cargo test` or `bx test` **Current: 34 tests passing** (All
phases 1-5 complete)

### History (`history/`)

Documentation archive for significant changes:

- **Bug Fixes**: Detailed documentation of production bugs and their fixes
- **Feature Implementations**: In-depth documentation of major new features
- **Naming Convention**: `YYYY-MM-DD_BUGFIX_<descriptive_name>.md` or
  `YYYY-MM-DD_FEATURE_<descriptive_name>.md`
- **Purpose**: Preserve detailed technical context that goes beyond CHANGELOG.md
  entries

See "Recording Bug Fixes and Features" section below for creation guidelines.

### Agent Documentation (`agent/`)

#### Core Documents

- `BIG_PICTURE_PLAN.md` - Future phases (6-9) and architectural reference
  - **Consult this for**: Future work, distribution targets, technical
    challenges, optional enhancements
  - **Note**: Completed phases (1-5) are documented in
    history/2025-10-25_INITIAL_IMPLEMENTATION.md

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

**Current Phase**: Phase 5 (Advanced Features) - COMPLETE

**Completed**:

- ✅ Phase 1: Basic Process Spawning
- ✅ Phase 2: Signal Handling
- ✅ Phase 3: Interactive Hotkey
- ✅ Phase 4: PTY-Based Testing
- ✅ Phase 5: Advanced Features (colored logging, --silent flag, restart
  debouncing)

**Next Steps**: See [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) for
Phase 6-9 (future work)

See
[history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)
for detailed implementation notes on completed phases.

## Architecture Overview

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
- Tests organized by phase in separate files
- Clean test output without raw mode artifacts
- Coverage: process spawning, signals, hotkeys, output forwarding, debouncing

**Manual Testing:**

- Long-running processes
- Processes that exit immediately
- Continuous output
- Stubborn processes ignoring SIGTERM
- Rapid restart requests (debouncing)

## Making Changes

### Before Implementing

1. Check implementation history:
   [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)
2. For future work, review
   [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md)
3. Look at existing tests in `tests/cli_phase*_tests.rs` for patterns

### After Implementing

1. Write integration tests (see
   [agent/actions/writing_integration_tests.md](agent/actions/writing_integration_tests.md))
2. Run `bx test` to ensure all tests pass
3. Run `bx lint` and `bx fmt` for code quality
4. For significant bug fixes or features, create history documentation (see
   below)
5. Update test counts in this file if new tests were added

## Recording Bug Fixes and Features

### When to Create History Documentation

Create a detailed history document for:

- **Production Bugs**: Any bug that affected users in production
- **Major Features**: Significant new functionality that changes user experience
- **Architecture Changes**: Technical improvements that affect multiple modules
- **Breaking Changes**: Any changes that require user action or change behavior

**Do NOT create history documents for**:

- Minor refactoring without user impact
- Documentation-only changes
- Small bug fixes in unreleased features
- Routine maintenance tasks

### History Document Format

**File naming**: `history/YYYY-MM-DD_TYPE_descriptive_name.md`

- `TYPE`: Either `BUGFIX` or `FEATURE`
- `descriptive_name`: Short, snake_case description
- Examples:
  - `2025-10-25_BUGFIX_process_group_cleanup.md`
  - `2025-11-15_FEATURE_config_file_support.md`

**Document structure**:

```markdown
# [Type]: [Short Title]

## Problem (for bugs) / Motivation (for features)

Clear description of what was wrong or why the feature was needed.

## Root Cause (for bugs) / Design (for features)

Technical explanation of the underlying issue or design decisions.

## Solution / Implementation

Detailed technical description of the fix/feature:

1. Code changes made
2. Files modified
3. Key implementation details

### Code Changes

Show relevant code snippets with file paths and explanations.

## Testing

Description of tests created:

- Test file names and locations
- What each test verifies
- Any special test setup or considerations

## Results

- Test results (number of tests passing)
- Performance impact (if applicable)
- Verification steps

## Technical Details

Additional context that helps understand the solution:

- Platform-specific considerations
- Dependencies involved
- Related issues or PRs

## Verification (optional)

Manual steps to verify the fix/feature works correctly.

## CHANGELOG Entry

The exact entry added to CHANGELOG.md for this change.
```

### Creating History Documentation - Checklist

When creating history documentation:

1. ✅ Create document in `history/` folder with proper naming
2. ✅ Include all relevant code snippets with file paths
3. ✅ Document all test files created or modified
4. ✅ Show test results proving the fix/feature works
5. ✅ Explain technical context and decisions
6. ✅ Reference the CHANGELOG.md entry
7. ✅ Use markdown code blocks with proper syntax highlighting
8. ✅ Include clear section headers for easy navigation

### Updating Related Documentation

When creating history documentation, also update:

1. **CHANGELOG.md**: Add entry under `[Unreleased]` or appropriate version
2. **README.md**: Update if user-facing behavior changed
3. **AGENT.md** (this file): Update test counts or implementation status if
   needed
4. **Test files**: Ensure new tests have clear documentation comments

### Example Reference

See `history/2025-10-25_BUGFIX_process_group_cleanup.md` for a complete example
of bug fix documentation.

## Dependencies

See `Cargo.toml` for full dependency list. Key libraries:

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

## Platform Support

**Target Platforms**:

- macOS (Apple Silicon without Intel)
- Linux (glibc & musl)

**Limitation**: Unix-only (signals not available on Windows)
