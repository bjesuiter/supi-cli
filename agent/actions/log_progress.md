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
### Next Action
Specific next steps:
1. What to implement
2. Where to implement it
3. Key technical approach

**Test with:** command to verify
```

## Keep it Short

- No repetition of BIG_PICTURE_PLAN.md content
- Only status, what changed, what's next
- 30-50 lines max per session

## Test Commands Section

Maintain a "Test Commands Per Phase" section at the end of PROGRESS.md. For each
completed phase, add specific test commands using `bx run`:

**Include:**

- Simple test (basic functionality)
- Edge case test (stderr, flags, etc.)
- Complex test (multi-line output, long-running)
- Relevant bx shortcuts (bx dev, bx dev-sleep)

**Mark future phases with:** `- TODO`

**Example test commands:**

```bash
# Simple
bx run echo "test"

# Complex
bx run bash -- -c 'for i in {1..5}; do echo $i; sleep 1; done'

# With flags
bx run --stop-on-child-exit echo "test"
```

## Example Entry

```markdown
## 2025-10-25: Session 2

### Status

**Phase 2 (Process Spawning): ✅ DONE**

- ProcessManager spawns child with tokio::process::Command
- Stdout/stderr forwarding working
- Process cleanup on exit
- Tested with echo and long-running processes

**Next: Phase 3 (Signal Handling)**

### What's Implemented
```

✅ src/process.rs - spawn/shutdown with output forwarding ✅ src/supervisor.rs -
basic run loop ⏸️ src/signals.rs - STUB

```
### Next Action
Implement signal handlers:
1. Add SIGINT/SIGTERM/SIGQUIT handlers using signal-hook-tokio
2. Forward signals to child process
3. Implement graceful shutdown with 5s timeout

**Test with:** `bx dev-sleep` then Ctrl+C
```
