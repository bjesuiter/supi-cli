# Agent Guide - Supi CLI Repository

This document helps AI agents navigate the repository structure and understand
where to find different types of information.

## Quick Navigation

- **User Documentation**: [README.md](README.md) - End-user facing documentation
- **Architecture & Implementation Plan**:
  [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md)
- **Current Progress**: [agent/PROGRESS.md](agent/PROGRESS.md)
- **Agent Guidelines**: [agent/actions/](agent/actions/)
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

- `cli_tests.rs` - Integration tests for all implemented features
  - Run with: `cargo test` or `bx test`
  - Currently 13 tests covering Phases 1-3
- `cli_bugfix_*.rs` - Specific test files for bug fixes with descriptive names
  - Example: `cli_bugfix_process_group_cleanup.rs`

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
5. For significant bug fixes or features, create history documentation (see
   below)

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
