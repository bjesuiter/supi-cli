# supi - Simple Process Supervisor

A lightweight CLI tool for supervising and managing arbitrary processes with
easy restart capabilities.

## Overview

`supi` is a simple process supervisor that spawns and manages child processes.
It allows you to restart processes on-demand using hotkeys or Unix signals,
making it ideal for development workflows where you need to frequently restart
services.

## Features

- **Process Management**: Spawns and supervises arbitrary commands
- **Signal Handling**: Responds to Unix signals for graceful shutdown and
  restart
- **Output Forwarding**: Forwards child process stdout and stderr in real-time
- **Interactive Restart**: Press a key to instantly restart your process
- **Flexible Configuration**: Customize restart signals and hotkeys

## Core Behavior

### Output Forwarding

- All child process output (stdout and stderr) is forwarded as raw as possible
- No buffering or modification of child output
- Input is NOT forwarded to the child process by default (an "interactive" mode
  might be added in the future)

### Signal Handling

- **Restart Signal**: `SIGUSR1` (default) - Restarts the child process
- **Stop Signals**: Responds to standard termination signals (SIGTERM, SIGINT,
  etc.)
- Gracefully terminates child process before exiting

### Interactive Control

- Press the `r` key (default) to restart the child process
- Terminal must be focused for hotkey to work (not a global hotkey)

### Child Process Exit

- By default, supi continues running even if the child process exits
- Allows you to restart the process using signals or hotkeys
- Can be configured to exit when child exits using `--stop-on-child-exit`

## Usage

```bash
# Basic usage
supi <command> [args...]

# Example: Run a development server
supi npm run dev

# Example: Run a Rust application
supi cargo run

# Stop supi when child exits
supi --stop-on-child-exit ./my-script.sh
```

## Command Line Options

### `--stop-on-child-exit`

**Default**: `false`

When enabled, supi will exit if the child process exits. When disabled
(default), supi continues running and you can restart the process using the
restart signal or hotkey.

```bash
supi --stop-on-child-exit npm start
```

### `--restart-signal <SIGNAL>`

**Default**: `SIGUSR1`

Specifies which Unix signal should trigger a process restart.

```bash
supi --restart-signal SIGUSR2 ./my-app
```

### `--restart-hotkey <KEY>`

**Default**: `r`

Specifies which key should trigger a process restart when pressed. Only works
when the terminal running supi is focused.

```bash
supi --restart-hotkey R ./my-app
```

## Example Workflows

### Development Server with Quick Restart

```bash
# Start your dev server, press 'r' to restart anytime
supi npm run dev
```

### Production-like Supervisor

```bash
# Exit when the main process exits
supi --stop-on-child-exit ./production-app
```

### Custom Signal Integration

```bash
# Use SIGUSR2 for restart
supi --restart-signal SIGUSR2 python app.py

# In another terminal, send restart signal
kill -SIGUSR2 $(pgrep -f "supi python")
```

## Installation

```bash
cargo install --path .
```

## Requirements

- Unix-like operating system (Linux, macOS)
- Rust 1.86 or higher

## Distribution Targets

Pre-built binaries are available for:

- `aarch64-apple-darwin` - Apple Silicon macOS
- `x86_64-unknown-linux-gnu` - Linux with glibc
- `x86_64-unknown-linux-musl` - Linux static binary (portable)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
