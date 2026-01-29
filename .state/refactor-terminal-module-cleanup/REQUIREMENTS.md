# Requirements: Clean up terminal module backward-compatibility shim

## Problem Statement
After the scroll region fix refactoring (PR #73), the terminal emulator was properly moved to `src/terminal/`, but a backward-compatibility shim was left behind in `src/player/terminal.rs`. This creates a confusing re-export chain:

```
src/terminal/           <- canonical location (actual implementation)
src/player/terminal.rs  <- shim re-exporting from crate::terminal
src/lib.rs              <- terminal_buffer module re-exporting from player::terminal
```

Several files still import via the old `terminal_buffer` alias rather than the canonical `terminal` module, creating unnecessary indirection and making the codebase harder to understand.

## Desired Outcome
- Single canonical import path for terminal types: `crate::terminal::{...}` (internal) or `agr::terminal::{...}` (external)
- No backward-compatibility shims or re-export chains
- All existing functionality preserved; this is a pure refactoring with no behavior changes

## Scope
### In Scope
- Delete `src/player/terminal.rs` (the 9-line re-export shim)
- Remove `pub mod terminal` and terminal re-exports from `src/player/mod.rs`
- Remove `terminal_buffer` module from `src/lib.rs`
- Update imports in `src/tui/widgets/file_explorer.rs` (3 occurrences)
- Update imports in `tests/integration/snapshot_tui_test.rs` (3 occurrences)
- Ensure `TerminalBuffer` re-export in `src/lib.rs` line 20 still works (update path if needed)

### Out of Scope
- Any changes to the terminal emulator implementation itself
- Changes to the `src/terminal/` module structure
- Adding new features or capabilities

## Verification Requirements
**Before making any changes:**
1. Scan the entire codebase for ALL usages of old import paths (`terminal_buffer`, `player::terminal`)
2. Document every file that needs to be updated
3. Run the full test suite and confirm all tests pass (baseline)

**After making changes:**
1. Re-scan the codebase to ensure no old import paths remain
2. Verify no files were missed during the update
3. Run the full test suite multiple times (at least 3 runs) to ensure no flaky failures

## Test Safety Requirements
- **DO NOT remove any existing tests** - only update import paths within them
- **DO NOT add any new tests** - this is a pure refactoring task
- **Only modify test files to update import paths** - no other changes to test logic
- Tests are the safety net; changing them defeats the purpose of regression detection

## Careful Execution Guidelines
1. Make changes incrementally, one file at a time
2. Run `cargo check` after each file modification
3. Run `cargo test` after completing all changes
4. Run the full test suite at least 3 times to catch any intermittent issues
5. If any test fails, investigate thoroughly before proceeding

## Acceptance Criteria
### Code Changes
- [ ] `src/player/terminal.rs` file is deleted
- [ ] `src/player/mod.rs` no longer has `pub mod terminal` or terminal type re-exports
- [ ] `src/lib.rs` no longer has `terminal_buffer` module
- [ ] All imports use `crate::terminal::` (internal) or `agr::terminal::` (external)
- [ ] Public API still exposes `TerminalBuffer` from `agr::` root (if currently exposed)

### Verification
- [ ] Pre-change codebase scan completed and documented
- [ ] Post-change codebase scan confirms no old import paths remain
- [ ] No functionality changes - only import path changes
- [ ] No test logic modified - only import paths within tests updated

### Build & Test
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (all existing tests)
- [ ] `cargo test` passes on repeated runs (at least 3 consecutive successful runs)
- [ ] `cargo clippy` passes without new warnings

### Test Integrity
- [ ] Test count before refactoring equals test count after refactoring
- [ ] No tests were removed
- [ ] No tests were added
- [ ] Test file changes are limited to import statements only

## Constraints
- Pure refactoring: no behavior changes
- Preserve public API compatibility (types that were publicly accessible should remain so)
- Test files: import path changes ONLY, no other modifications

## Context
- Related PR: #73 (scroll region fix that moved terminal to `src/terminal/`)
- The `src/terminal/` module is the canonical location containing the full implementation
- This cleanup was identified during review of the module structure

---
**Sign-off:** Approved by user
