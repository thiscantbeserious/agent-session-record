# Requirements: Refactor Native Player Modules

**Branch:** refactor-native-player-modules
**Sign-off:** Approved by user

---

## Problem Statement

The native player implementation (`src/player/native.rs`) is a monolithic 2,481-line file containing all player functionality: rendering, input handling, state management, UI components, and ANSI color conversion. This makes the code difficult to maintain, test, and extend.

Additionally, the legacy `asciinema.rs` module provides a CLI wrapper that shells out to the asciinema binary. This module is no longer used in the codebase - the native player is now the default and only player used by all commands.

## Goals

1. **Maintainability** - Split `native.rs` into logical, focused modules
2. **Testability** - Increase test coverage through better separation of concerns
3. **Code Cleanup** - Remove unused legacy code (`asciinema.rs`)
4. **Stability** - Prevent regressions through comprehensive snapshot testing

## Scope

### In Scope

1. **Refactor `src/player/native.rs`** - split the monolithic file into a modular structure that improves maintainability and testability. The refactoring should scope functionality into logical areas such as shortcut/input management, rendering, state management, and similar concerns.

2. **Remove `src/player/asciinema.rs`** including:
   - The module file itself
   - Module declaration in `src/player/mod.rs`
   - Public re-exports (`play_session_asciinema`, `play_session_with_speed`)
   - Any documentation references to the legacy asciinema CLI wrapper

3. **TUI Snapshot Testing** (Critical - BLOCKING PREREQUISITE):
   - **BEFORE refactoring begins:** Capture TUI snapshots of ALL player states as a baseline
   - **AFTER refactoring completes:** Verify all snapshots match the baseline exactly
   - No refactoring work may begin until baseline snapshots are captured and committed
   - No refactoring work is considered complete until snapshot verification passes
   - Player states to capture include:
     - Normal playback (playing/paused)
     - Help overlay visible
     - Viewport mode active
     - Free mode active
     - Progress bar with markers
     - Scroll indicators (all directions)
     - Status bar variations (different speeds, marker counts)
     - `agr ls` player integration states

4. **Increase test coverage** for newly modularized components

### Out of Scope

- Adding new player features
- Changing player behavior or controls
- Modifying the public API (`play_session`, `play_session_native`, `PlaybackResult`)
- Changes to other modules outside the player

## Acceptance Criteria

### Prerequisite (Must Complete First)

1. TUI snapshot baseline captured for ALL player states before any refactoring begins
2. Baseline snapshots committed to the repository as reference

### Functional Requirements

1. `agr play` command works identically before and after refactoring
2. `agr ls` player capabilities work identically before and after refactoring
3. All existing player controls function correctly:
   - **Playback:** Space (pause/resume), ←/→ (seek ±5s), Shift+←/→ (seek ±5%), +/- (speed), Home/End (start/end)
   - **Markers:** m (jump to next marker)
   - **Free Mode:** f (toggle), ↑/↓ (move highlight), Esc (exit)
   - **Viewport:** v (toggle), ↑↓←→ (scroll), r (resize to recording), Esc (exit)
   - **General:** ? (help), q (quit)
4. All existing tests pass
5. No visual regressions (verified by snapshot comparison)

### Architectural Requirements

1. `native.rs` refactored from a monolithic file into a modular structure
2. Each module has a clear, single responsibility
3. Public API unchanged: `play_session`, `play_session_native`, `PlaybackResult`
4. No dead code from `asciinema.rs` remains

### Quality Requirements

1. New tests added for modularized components
2. Test coverage maintained or improved
3. TUI snapshot tests pass - post-refactor output matches pre-refactor baseline exactly
4. All CI checks pass (clippy, tests, formatting)

## Constraints

- Breaking changes are acceptable (no users/releases yet)
- Must not change visible player behavior
- Must preserve all keyboard shortcuts and mouse interactions

---

## Sign-off

**Status:** Approved by user

Once approved, this document will guide the Architect in designing the implementation approach.
