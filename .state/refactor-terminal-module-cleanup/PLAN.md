# Execution Plan: Terminal Module Re-export Chain Cleanup

## Overview
Remove unnecessary re-export chain for `TerminalBuffer` using Option A (Outside-In approach).

## Approach
Update consumers first, then remove re-exports working inward toward the source.

---

## Stage 1: Pre-verification (Baseline)

### Objective
Establish baseline test counts and ensure clean starting state.

### Actions
1. Run `cargo test` and record total test count
2. Run `cargo clippy` and verify no warnings
3. Document baseline metrics

### Verification
- [ ] All tests pass
- [ ] Clippy reports no warnings
- [ ] Baseline test count recorded

### Rollback
N/A - read-only stage

---

## Stage 2: Update file_explorer.rs Imports

### Objective
Change internal crate imports from `crate::terminal_buffer` to `crate::terminal`.

### Files Modified
- `src/tui/widgets/file_explorer.rs`

### Changes
| Line | Old | New |
|------|-----|-----|
| 82 | `crate::terminal_buffer::TerminalBuffer` | `crate::terminal::TerminalBuffer` |
| 118 | `crate::terminal_buffer::TerminalBuffer` | `crate::terminal::TerminalBuffer` |
| 244 | `crate::terminal_buffer::TerminalBuffer` | `crate::terminal::TerminalBuffer` |

### Verification
- [ ] `cargo check` succeeds
- [ ] `cargo test --lib` passes

### Rollback
```bash
git checkout src/tui/widgets/file_explorer.rs
```

---

## Stage 3: Update snapshot_tui_test.rs Imports

### Objective
Change test imports from `agr::terminal_buffer` to `agr::terminal`.

### Files Modified
- `tests/integration/snapshot_tui_test.rs`

### Changes
| Line | Old | New |
|------|-----|-----|
| 579 | `agr::terminal_buffer::TerminalBuffer` | `agr::terminal::TerminalBuffer` |
| 624 | `agr::terminal_buffer::TerminalBuffer` | `agr::terminal::TerminalBuffer` |
| 652 | `agr::terminal_buffer::TerminalBuffer` | `agr::terminal::TerminalBuffer` |

### Verification
- [ ] `cargo check --tests` succeeds
- [ ] `cargo test snapshot_tui` passes

### Rollback
```bash
git checkout tests/integration/snapshot_tui_test.rs
```

---

## Stage 4: Update src/lib.rs TerminalBuffer Re-export Path

### Objective
Change the `TerminalBuffer` re-export to come directly from `terminal` module instead of through `player`.

### Files Modified
- `src/lib.rs`

### Changes
| Line | Old | New |
|------|-----|-----|
| 20 | `pub use player::TerminalBuffer;` | `pub use terminal::TerminalBuffer;` |

### Verification
- [ ] `cargo check` succeeds
- [ ] `cargo test` passes (full suite)

### Rollback
```bash
git checkout src/lib.rs
```

---

## Stage 5: Remove terminal_buffer Module from src/lib.rs

### Objective
Remove the deprecated `terminal_buffer` module alias that re-exported from player.

### Files Modified
- `src/lib.rs`

### Changes
Remove lines 25-28:
```rust
// Remove this block:
pub mod terminal_buffer {
    pub use crate::player::TerminalBuffer;
}
```

### Verification
- [ ] `cargo check` succeeds
- [ ] `cargo test` passes (full suite)
- [ ] No references to `terminal_buffer` module remain in crate

### Rollback
```bash
git checkout src/lib.rs
```

---

## Stage 6: Remove terminal from src/player/mod.rs

### Objective
Remove the terminal submodule and its re-exports from the player module.

### Files Modified
- `src/player/mod.rs`

### Changes
| Line | Action |
|------|--------|
| 24-25 | Remove `mod terminal;` and `pub mod terminal;` |
| 29-31 | Remove `pub use terminal::TerminalBuffer;` and related re-exports |

### Verification
- [ ] `cargo check` succeeds
- [ ] `cargo test` passes (full suite)

### Rollback
```bash
git checkout src/player/mod.rs
```

---

## Stage 7: Delete src/player/terminal.rs

### Objective
Remove the now-unused terminal wrapper module.

### Files Modified
- `src/player/terminal.rs` (DELETE)

### Changes
Delete the entire file.

### Verification
- [ ] `cargo check` succeeds
- [ ] `cargo test` passes (full suite)
- [ ] File no longer exists

### Rollback
```bash
git checkout src/player/terminal.rs
```

---

## Stage 8: Post-verification (Final)

### Objective
Comprehensive verification that refactoring is complete and correct.

### Actions
1. Run `cargo test` three times to ensure stability
2. Run `cargo clippy -- -D warnings`
3. Run `cargo doc --no-deps` to verify documentation builds
4. Scan codebase for any remaining `terminal_buffer` references
5. Verify `TerminalBuffer` is accessible via `agr::terminal::TerminalBuffer`

### Verification Commands
```bash
# Test stability (run 3 times)
cargo test
cargo test
cargo test

# Clippy
cargo clippy -- -D warnings

# Documentation
cargo doc --no-deps

# Scan for old imports
grep -r "terminal_buffer" src/ tests/
```

### Success Criteria
- [ ] All tests pass consistently (3 runs)
- [ ] Clippy reports no warnings
- [ ] Documentation builds successfully
- [ ] No `terminal_buffer` references in codebase
- [ ] Test count matches baseline from Stage 1

---

## Summary of Changes

| File | Action | Lines Affected |
|------|--------|----------------|
| `src/tui/widgets/file_explorer.rs` | Modify | 82, 118, 244 |
| `tests/integration/snapshot_tui_test.rs` | Modify | 579, 624, 652 |
| `src/lib.rs` | Modify | 20, 25-28 |
| `src/player/mod.rs` | Modify | 24-25, 29-31 |
| `src/player/terminal.rs` | Delete | Entire file |

---

## Progress

| Stage | Description | Status | Verified |
|-------|-------------|--------|----------|
| 1 | Pre-verification (Baseline) | Complete | [x] |
| 2 | Update file_explorer.rs imports | Complete | [x] |
| 3 | Update snapshot_tui_test.rs imports | Complete | [x] |
| 4 | Update src/lib.rs re-export path | Complete | [x] |
| 5 | Remove terminal_buffer module | Complete | [x] |
| 6 | Remove terminal from player/mod.rs | Complete | [x] |
| 7 | Delete src/player/terminal.rs | Complete | [x] |
| 8 | Post-verification (Final) | Complete | [x] |

---

## Notes
- Each stage includes explicit rollback instructions
- Verification uses `cargo check` for fast feedback
- Full test suite run at critical stages (4, 5, 6, 7, 8)
- Stage 8 includes stability testing with 3 test runs
