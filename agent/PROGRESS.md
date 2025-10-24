# Implementation Progress

## 2025-10-24: Session 1

### Status

**Phase 1 (CLI & Skeleton): ✅ DONE**

- All modules created with stubs
- CLI fully functional (clap with all args)
- All dependencies added to Cargo.toml
- Compiles successfully
- All functions stubbed with `anyhow::bail!("not implemented yet")`

**Next: Phase 2 (Basic Process Spawning)**

### What's Implemented

```
✅ src/cli.rs        - Complete CLI parsing
✅ src/main.rs       - Entry point wiring
⏸️  src/process.rs   - STUB (spawn/restart/shutdown)
⏸️  src/supervisor.rs - STUB (run method)
⏸️  src/signals.rs   - STUB
⏸️  src/hotkey.rs    - STUB
```

### Next Action

Implement `src/process.rs`:

1. `spawn()` - Use `tokio::process::Command`, capture stdout/stderr
2. Forward output line-by-line with `tokio::io::BufReader`
3. `shutdown()` - Kill child process
4. Track process state (PID, running status)

Then update `supervisor.rs` to call spawn and wait for exit.

**Test with:** `bx dev` or `cargo run -- echo hello`
