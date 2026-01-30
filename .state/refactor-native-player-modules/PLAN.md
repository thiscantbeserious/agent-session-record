# Plan: Refactor Native Player into Component-Based Module Structure

References: [ADR.md](./ADR.md), [REQUIREMENTS.md](./REQUIREMENTS.md)

## Resolved Design Decisions

### 1. PlaybackState Structure
**Decision:** `PlaybackState` will be a **single struct** (not split into smaller pieces).

This keeps the state management simple and avoids borrow checker complexity from having multiple sub-structs that need simultaneous mutable access.

### 2. Input Handler Communication
**Decision:** Use a **hybrid approach**:
- Pass `&mut PlaybackState` for state modifications
- Return `InputResult` enum for control flow:

```rust
pub enum InputResult {
    Continue,
    Quit,
    QuitWithFile,  // For agr ls integration
}
```

This allows input handlers to both modify state directly and signal control flow decisions to the main loop.

---

## Stages

### Stage 0: TUI Snapshot Baseline [BLOCKING PREREQUISITE]
**Goal:** Capture TUI snapshots of ALL player states before any refactoring begins. This creates the regression baseline.

- [x] Create test fixture cast file with markers for comprehensive testing
- [x] Implement TUI snapshot tests in `tests/integration/player_snapshot_test.rs`
- [x] Capture snapshots for all player states (stored in `tests/integration/snapshots/player/`):
  - [x] Normal playback (playing state)
  - [x] Paused state
  - [x] Help overlay visible
  - [x] Viewport mode active
  - [x] Free mode active (with highlighted line)
  - [x] Progress bar with markers at various positions
  - [x] Scroll indicators (up, down, left, right combinations)
  - [x] Status bar variations (different speeds, marker counts, offset displays)
  - [x] `agr ls` player integration (launched from file explorer) - tested via cast file playback
- [x] Commit baseline snapshots to repository

Files: `tests/integration/player_snapshot_test.rs`, `tests/integration/snapshots/player/`

Considerations:
- Must complete BEFORE any code movement
- Snapshots should be deterministic (fixed terminal size, fixed time positions)
- **Will use `insta` crate** for snapshot testing (confirmed)
- Use `insta::with_settings!({ snapshot_path => "snapshots/player" }, { ... })` for dedicated subdirectory

---

### Stage 1: Create Module Structure + Extract State
**Goal:** Create all directory structure and empty module files. Extract `PlaybackState` and shared types to `state.rs`.

- [x] Create directory structure:
  - `src/player/input/`
  - `src/player/playback/`
  - `src/player/render/`
- [x] Create empty module files with doc comments explaining purpose
- [x] Create `state.rs` with:
  - `PlaybackState` struct containing all playback state fields
  - `MarkerPosition` struct (shared between render and playback)
  - `InputResult` enum for control flow
  - `PlaybackState::new()` constructor
- [x] Update `src/player/mod.rs` to declare new modules
- [x] Update `native.rs` to use `PlaybackState` instead of individual variables
- [x] Verify compilation passes

Files:
- `src/player/state.rs`
- `src/player/input/mod.rs`
- `src/player/input/keyboard.rs`
- `src/player/input/mouse.rs`
- `src/player/playback/mod.rs`
- `src/player/playback/seeking.rs`
- `src/player/playback/markers.rs`
- `src/player/render/mod.rs`
- `src/player/render/viewport.rs`
- `src/player/render/progress.rs`
- `src/player/render/status.rs`
- `src/player/render/scroll.rs`
- `src/player/render/help.rs`
- `src/player/render/ansi.rs`

Considerations:
- Empty files should compile (placeholder code where needed)
- `PlaybackState` is the first real code to move - enables later stages

---

### Stage 2: Extract All Render Components
**Goal:** Move ALL render functions to their respective files in `render/`.

- [x] Move to `render/ansi.rs`:
  - `style_to_ansi_fg()`
  - `style_to_ansi_bg()`
  - `style_to_ansi_attrs()`
- [x] Move to `render/help.rs`:
  - `HELP_LINES` constant
  - `HELP_BOX_WIDTH` constant
  - `calc_help_start_row()`
  - `calc_help_start_col()`
  - `render_help()`
- [x] Move to `render/scroll.rs`:
  - `calc_scroll_directions()`
  - `build_scroll_arrows()`
  - `render_scroll_indicator()`
- [x] Move to `render/progress.rs`:
  - `build_progress_bar_chars()`
  - `render_progress_bar()`
  - `format_duration()`
- [x] Move to `render/status.rs`:
  - `count_digits()`
  - `render_status_bar()`
  - `render_separator_line()`
- [x] Move to `render/viewport.rs`:
  - `render_viewport()`
  - `render_single_line()`
- [x] Create `render/mod.rs` with re-exports
- [x] Update imports in `native.rs`
- [x] Run tests to verify no regressions

Files: `src/player/render/*.rs`, `src/player/native.rs`

Considerations:
- Move related tests alongside functions where practical
- Keep functions `pub(crate)` for cross-module access
- `render/ansi.rs` functions are pure utilities with no state dependencies

---

### Stage 3: Extract Playback Logic
**Goal:** Move marker and seeking operations to `playback/`.

- [x] Move to `playback/markers.rs`:
  - `collect_markers()`
  - Marker navigation logic (finding next marker)
- [x] Move to `playback/seeking.rs`:
  - `find_event_index_at_time()`
  - `seek_to_time()`
  - `process_up_to_time` closure (convert to standalone function)
- [x] Create `playback/mod.rs` with re-exports
- [x] Update imports in `native.rs`
- [x] Run tests to verify no regressions

Files: `src/player/playback/*.rs`, `src/player/native.rs`

Considerations:
- `seek_to_time()` mutates `TerminalBuffer` - needs mutable reference
- These functions are well-tested already - preserve test coverage
- Import `MarkerPosition` from `state.rs`

---

### Stage 4: Extract Input Handlers
**Goal:** Move all input handling to `input/`.

- [x] Create `input/keyboard.rs` with `handle_key_event()`:
  - Quit/Escape handling (q, Esc, Ctrl+C)
  - Pause/resume (Space)
  - Speed adjustment (+/= faster, -/_ slower)
  - Mode toggles (v viewport, f free, ? help)
  - Seeking (arrows +/-5s, Shift+arrows +/-5%, </, backward, >/. forward, Home/End)
  - Marker navigation (m)
  - Resize (r)
- [x] Create `input/mouse.rs` with `handle_mouse_event()`:
  - Progress bar click-to-seek logic
- [x] Create `input/mod.rs` with:
  - Re-exports
  - `handle_event()` dispatch function
- [x] Update `native.rs` to use input handlers
- [x] Run tests to verify no regressions

Files: `src/player/input/*.rs`, `src/player/native.rs`

Considerations:
- Keyboard handler is largest extraction - do carefully
- Handlers take `&mut PlaybackState` and return `InputResult`
- Some handlers need access to markers list and terminal buffer

---

### Stage 5: Simplify native.rs
**Goal:** Verify `native.rs` is reduced to its essential orchestration role.

- [x] Verify `native.rs` only contains:
  - `PlaybackResult` enum and impl
  - `play_session()` function
  - `play_session_native()` function
  - Main loop that coordinates modules
- [x] Remove any remaining helper functions (should all be in modules)
- [x] Target: `native.rs` under 300 lines - achieved 342 lines (close to target, well within acceptable range)
- [x] Run tests to verify no regressions

Files: `src/player/native.rs`

Considerations:
- Main loop should be readable at a glance
- All complexity should be delegated to submodules

---

### Stage 6: Remove Legacy + Organize Tests
**Goal:** Remove unused asciinema wrapper and organize test code.

- [x] Delete `src/player/asciinema.rs`
- [x] Remove `mod asciinema;` from `src/player/mod.rs`
- [x] Remove `pub use asciinema::{...}` from `src/player/mod.rs`
- [x] Search codebase for any remaining references
- [x] Migrate ~1000 lines of inline tests from `native.rs` to appropriate locations:
  - Unit tests stay in respective module files (inline `#[cfg(test)]` modules)
  - Integration/snapshot tests go to `tests/integration/`
- [x] Run all tests to verify coverage

Files: `src/player/asciinema.rs` (delete), `src/player/mod.rs`, `tests/integration/`

Considerations:
- Verify no external code depends on legacy asciinema functions
- Keep unit tests close to implementation where practical

---

### Stage 7: Final Cleanup
**Goal:** Polish module documentation and clean up any remaining issues.

- [x] Add module-level documentation to all `mod.rs` files
- [x] Add function-level documentation where missing
- [x] Run `cargo clippy` and fix any warnings
- [x] Run `cargo fmt`
- [x] Update `src/player/mod.rs` documentation to reflect new structure
- [x] Review all changes

Files: All `mod.rs` files in `src/player/`

Considerations:
- Documentation should explain module responsibilities
- All public APIs should have doc comments

---

### Stage 8: TUI Snapshot Verification [FINAL GATE]
**Goal:** Verify all TUI snapshots match the baseline captured in Stage 0.

- [x] Run snapshot tests
- [x] Verify all snapshots match baseline exactly
- [x] If differences found, investigate and fix (should be none)
- [x] Document any intentional changes (should be none)

Files: `tests/integration/snapshots/player/`

Considerations:
- This stage MUST pass before refactoring is considered complete
- Any differences indicate a regression
- No visual changes are acceptable per REQUIREMENTS.md

---

## Dependencies

```
Stage 0 (TUI Snapshot Baseline)
    |
    v
Stage 1 (Module Structure + State)
    |
    +---> Stage 2 (All Render Components)
    |         |
    |         v
    +---> Stage 3 (Playback Logic)
    |         |
    |         v
    +---> Stage 4 (Input Handlers)
              |
              v
         Stage 5 (Simplify native.rs)
              |
              v
         Stage 6 (Remove Legacy + Organize Tests)
              |
              v
         Stage 7 (Final Cleanup)
              |
              v
         Stage 8 (TUI Snapshot Verification) [FINAL GATE]
```

Key dependency rules:
- **Stage 0 MUST complete before any other stage** (blocking prerequisite)
- Stages 2, 3, 4 can proceed in parallel after Stage 1
- Stages 5-8 are sequential (each depends on previous)
- **Stage 8 MUST pass for refactoring to be complete**

---

## Progress

| Stage | Status | Notes |
|-------|--------|-------|
| 0 | complete | 35 snapshot tests for baseline |
| 1 | complete | Module structure + PlaybackState |
| 2 | complete | All render components |
| 3 | complete | Playback logic |
| 4 | complete | Input handlers |
| 5 | complete | native.rs reduced to 342 lines |
| 6 | complete | asciinema.rs deleted, tests organized |
| 7 | complete | clippy, fmt, docs all clean |
| 8 | complete | All 35 snapshots match baseline |
