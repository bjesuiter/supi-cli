# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-24

### Added

- Published to crates.io - first stable release! 🎉
- Complete CLI documentation in README for all 7 command-line flags
- Publishing workflow automation via `bx dry` command
- Publishing guide for maintainers in README
- Comprehensive installation instructions with crates.io as primary method

### Changed

- **Binary name changed from `supi-cli` to `supi`** for cleaner command-line
  usage
- CLI output now shows `supi` instead of `supi-cli` in version and help
- Edition changed from 2024 to 2021 for broader Rust version compatibility
  (1.56+)
- Test version assertions now dynamically read from `CARGO_PKG_VERSION`
- Updated all test files to reference `supi` binary name

### Fixed

- Tests no longer hardcode version numbers - automatically sync with Cargo.toml

### Package Metadata

- Added homepage, readme, and exclude fields to Cargo.toml for crates.io
- Excluded development files (agent/, AGENT.md, bonnie.toml) from package
- Binary explicitly named "supi" via `[[bin]]` section

### Documentation

- Complete documentation of all CLI flags in README:
  - `--stop-on-child-exit`
  - `--restart-signal`
  - `--restart-hotkey`
  - `--restart-debounce-ms`
  - `--log-color`
  - `--info-color`
  - `--silent`
- Added publishing information section for maintainers
- Updated development information section

### Technical

- Package size: 22 files, ~30KB compressed
- All 34 tests passing with new binary name
- Ready for `cargo install --locked supi-cli`

[1.0.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v1.0.0

## [0.2.0] - 2025-10-24

### Added

- Colored logging for supervisor messages via `--log-color` flag (default:
  yellow)
- Separate color control for informational messages via `--info-color` flag
  (default: green)
- Support for multiple color options: yellow, red, green, blue, cyan, magenta,
  white, none
- `--silent` flag to suppress all supervisor output while preserving child
  process output
- Restart debouncing via `--restart-debounce-ms` option (default: 1000ms, set to
  0 to disable)
- Configuration display in info messages showing configured hotkey and restart
  signal
- "Child process running (PID: xxx)" log message after spawn for better
  visibility

### Changed

- Refactored output system from function-based to stateful `Output` struct
- Output struct now encapsulates `log_color`, `info_color`, and `silent` flag
- Clear separation between supervisor logs (suppressible) and child output
  (always visible)
- Improved informational messages to show actual configured values

### Technical

- Replaced macro-based output with `Output` struct methods throughout codebase
- Output struct is clonable and can be passed to spawned tasks
- Debouncing applies to both hotkey and signal restart triggers
- Added 12 new tests for Phase 5 features (34 tests total, up from 22)

### Features Summary

✅ Colored and customizable supervisor output\
✅ Silent mode for minimal output\
✅ Restart debouncing to prevent rapid restarts\
✅ Enhanced configuration visibility

[0.2.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v0.2.0

## [0.1.0] - 2025-10-24

### Added

- Process spawning and supervision with output forwarding (stdout/stderr)
- Signal handling: graceful shutdown via SIGTERM, SIGINT, SIGQUIT
- Configurable restart signals: SIGUSR1 (default), SIGUSR2, SIGHUP
- Interactive hotkey restart (default: 'r', configurable via `--restart-hotkey`)
- `--stop-on-child-exit` flag to exit supervisor when child process terminates
- Graceful shutdown with 5-second timeout before force kill
- Signal forwarding to child processes
- PTY-based integration testing (22 tests passing)

### Features Summary

✅ Basic process spawning and management\
✅ Signal handling for graceful shutdown and restart\
✅ Interactive hotkey restart with TTY fallback\
✅ Comprehensive test coverage with PTY support

[0.1.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v0.1.0
