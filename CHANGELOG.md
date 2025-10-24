# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
