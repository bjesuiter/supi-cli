# Agent Guide - Supi CLI Repository

This document helps AI agents navigate the repository and understand where to
find information.

## AI Agent Instructions

When working on this project:

- Implement step by step for review
- Answer concise, sacrifice grammar for brevity
- Check
  [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)
  for completed phases (1-5)
- Look at [README.md](README.md) for:
  - Usage instructions and CLI options
  - Installation and publishing info
  - User-facing roadmap ("Future Considerations")
- See [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) for:
  - Architecture and design decisions
  - Technical challenges and solutions
  - Distribution targets and platform considerations
- Write integration tests per
  [agent/actions/writing_integration_tests.md](agent/actions/writing_integration_tests.md)

## Quick Navigation

- **User Documentation**: [README.md](README.md) - Usage, installation,
  publishing, and roadmap
- **Architecture & Technical**:
  [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) - Design, challenges,
  dependencies
- **Completed Work**:
  [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md) -
  Phases 1-5 details
- **Change History**: [history/](history/) - Bug fixes and features
  (`YYYY-MM-DD_TYPE_name.md`)
- **Agent Guidelines**: [agent/actions/](agent/actions/) - Process docs for
  tests and logging
- **Dependencies**: [Cargo.toml](Cargo.toml) - All production and dev
  dependencies
- **Task Runner**: [bonnie.toml](bonnie.toml) - Available `bx` commands

## Repository Structure

**For detailed architecture**: See
[agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) (module structure,
dependencies, error handling, testing strategy)

### Quick Reference

- **`src/`** - Main source code (6 modules: main, cli, supervisor, process,
  signals, hotkey, output)
- **`tests/`** - Integration tests organized by phase + bug-specific tests (34
  tests total)
- **`history/`** - Detailed change documentation (`YYYY-MM-DD_BUGFIX_name.md` or
  `FEATURE_name.md`)
- **`agent/`** - Agent documentation (BIG_PICTURE_PLAN.md, actions/,
  bonnie.toml-reference.md)

## Development Workflow

### Common Commands

```bash
bx test        # Run all tests
bx fmt         # Format code
bx lint        # Run clippy
bx run         # Build and run in dev mode
bx release     # Build optimized release binary
```

See [bonnie.toml](bonnie.toml) for all commands.

## Implementation Status

**Current**: Phase 5 (Advanced Features) - COMPLETE

**Completed**: Phases 1-5 (process spawning, signals, hotkeys, PTY testing,
advanced features)

**Next Steps**: See [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) for
Phase 6-9

**Details**:
[history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)

## Making Changes

### Before

1. Check
   [history/2025-10-25_INITIAL_IMPLEMENTATION.md](history/2025-10-25_INITIAL_IMPLEMENTATION.md)
   for context
2. Review [agent/BIG_PICTURE_PLAN.md](agent/BIG_PICTURE_PLAN.md) for future work
3. Look at `tests/cli_phase*_tests.rs` for test patterns

### After

1. Write integration tests (see
   [agent/actions/writing_integration_tests.md](agent/actions/writing_integration_tests.md))
2. Run `bx test`, `bx lint`, `bx fmt`
3. For significant bugs/features, create history doc (see below)
4. Update CHANGELOG.md and README.md if user-facing changes
5. Update test counts in AGENT.md if new tests added

## Recording Changes

### When to Create History Documentation

**Create for:**

- Production bugs
- Major features
- Architecture changes
- Breaking changes

**Skip for:**

- Minor refactoring
- Documentation-only changes
- Small unreleased bug fixes

### History Document Guidelines

**Naming**: `history/YYYY-MM-DD_BUGFIX_name.md` or
`history/YYYY-MM-DD_FEATURE_name.md`

**Structure**: Problem/Motivation → Root Cause/Design → Solution → Testing →
Results → Technical Details → CHANGELOG Entry

**Example**: See `history/2025-10-25_BUGFIX_process_group_cleanup.md` for
complete template

**Remember to update**:

- CHANGELOG.md (under `[Unreleased]`)
- README.md (if user-facing)
- Test counts in AGENT.md
- Test file documentation comments
