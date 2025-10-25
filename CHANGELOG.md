# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Process tree cleanup**: Child processes spawned by supervised commands
  (e.g., `bash -c "npm run build && npm run dev"`) are now properly terminated
  when the supervisor shuts down or restarts

## [1.0.0] - 2025-10-24

### Added

- Published to crates.io - first stable release! 🎉
- Installable via `cargo install --locked supi-cli`
- Complete CLI documentation for all flags

### Changed

- **Binary name changed from `supi-cli` to `supi`** for cleaner command-line
  usage
- Improved Rust version compatibility (supports 1.56+)

[1.0.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v1.0.0

## [0.2.0] - 2025-10-24

### Added

- Colored logging with `--log-color` (default: yellow) and `--info-color`
  (default: green) flags
- Color options: yellow, red, green, blue, cyan, magenta, white, none
- `--silent` flag to suppress supervisor output while preserving child output
- Restart debouncing via `--restart-debounce-ms` (default: 1000ms, 0 to disable)
- Configuration display showing configured hotkey and restart signal
- PID display in log messages

[0.2.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v0.2.0

## [0.1.0] - 2025-10-24

### Added

- Process spawning and supervision with output forwarding
- Signal handling for graceful shutdown (SIGTERM, SIGINT, SIGQUIT)
- Configurable restart signals: SIGUSR1 (default), SIGUSR2, SIGHUP
- Interactive hotkey restart (default: 'r', configurable via `--restart-hotkey`)
- `--stop-on-child-exit` flag to exit when child process terminates
- Graceful shutdown with 5-second timeout before force kill

[0.1.0]: https://github.com/bjesuiter/supi-cli/releases/tag/v0.1.0
