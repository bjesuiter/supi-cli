# How to Log Progress

## When

After completing a phase or significant milestone from BIG_PICTURE_PLAN.md.

## Format

Add new session entry to `agent/PROGRESS.md`:

```markdown
## YYYY-MM-DD: Session N

### Status

**Phase X (Name): ✅ DONE** or **Phase X (Name): 🚧 IN PROGRESS**

- Brief bullets of what was accomplished
- Any blockers or issues

**Next: Phase Y (Name)**

### What's Implemented
```

✅ module/file - Description ✅ module/file - Description ⏸️ module/file - STUB
or partial ❌ module/file - Not started

```
### Tests Added (tests/filename.rs)
- `test_name` - Description
- `test_name` - Description

### Next Action
Specific next steps:
1. What to implement
2. Where to implement it
3. Key technical approach

Add tests: `test_name`, `test_name`
```

## Keep it Short

- No repetition of BIG_PICTURE_PLAN.md content
- Only status, what changed, what's next
- 30-50 lines max per session

## Integration Tests Section

For each phase, write integration tests in `tests/` directory. In PROGRESS.md,
list applicable tests for each phase.

**Test Guidelines:**

- Use `assert_cmd` for CLI testing
- Test files: `tests/cli_tests.rs`, `tests/signal_tests.rs`, etc.
- Add tests as you implement features
- Simple tests (basic functionality)
- Edge cases (errors, flags, special scenarios)
- Complex scenarios (multi-step, async behavior)

**In Progress Reports:** List tests by name, grouped by phase:

```markdown
### Tests for Phase X

- `test_feature_basic` - Basic functionality
- `test_feature_with_flag` - Flag behavior
- `test_feature_error_case` - Error handling
```

Run tests: `bx test` or `bx test -- test_name`

## Example Entry

```markdown
## 2025-10-25: Session 2

### Status

**Phase 2 (Process Spawning): ✅ DONE**

- ProcessManager spawns child with tokio::process::Command
- Stdout/stderr forwarding working
- Process cleanup on exit

**Next: Phase 3 (Signal Handling)**

### What's Implemented
```

✅ src/process.rs - spawn/shutdown with output forwarding ✅ src/supervisor.rs -
basic run loop ⏸️ src/signals.rs - STUB

```
### Tests Added (tests/cli_tests.rs)
- `test_simple_echo` - Basic process spawning
- `test_stdout_forwarding` - Multi-line output
- `test_stderr_forwarding` - Error output
- `test_stop_on_child_exit_flag` - Flag behavior

### Next Action
Implement signal handlers:
1. Add SIGINT/SIGTERM/SIGQUIT handlers using signal-hook-tokio
2. Forward signals to child process
3. Implement graceful shutdown with 5s timeout

Add tests: `test_sigterm_graceful_shutdown`, `test_sigint_graceful_shutdown`
```

## Update BIG_PICTURE_PLAN.md

Update BIG_PICTURE_PLAN.md to reflect the progress by ticking off all action
items that are done.
