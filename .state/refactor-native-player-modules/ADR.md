# ADR: Refactor Native Player into Component-Based Module Structure

## Status
**Accepted** - Ready for implementation

## Context

The native player implementation (`src/player/native.rs`) is a monolithic 2,481-line file containing all player functionality:
- Playback state management
- Input handling (keyboard and mouse events)
- UI rendering (viewport, progress bar, status bar, help overlay, scroll indicators)
- Seeking and marker navigation
- ANSI color conversion utilities

Additionally, `src/player/asciinema.rs` provides a legacy CLI wrapper that shells out to the asciinema binary. This module is unused - the native player is now the default for all commands.

This structure makes the code difficult to:
- Navigate and understand
- Test individual components in isolation
- Extend with new features
- Maintain without risk of regressions

## Options Considered

### Option 1: Flat Module Split
Split `native.rs` into multiple flat files in `src/player/`:
```
src/player/
  mod.rs
  state.rs
  input.rs
  render.rs
  seeking.rs
  ansi.rs
```

- Pros: Simple structure, fewer directories
- Cons: Files would still be large; input/render logic is complex enough to warrant further subdivision

### Option 2: Domain-Based Structure
Organize by domain concepts:
```
src/player/
  mod.rs
  playback/        # state, seeking, markers
  display/         # all rendering
  controls/        # all input
```

- Pros: Domain-aligned, intuitive navigation
- Cons: Mixes unrelated concerns (e.g., state struct + seeking logic); awkward fit for utility code like ANSI conversion

### Option 3: Component-Based Structure (Selected)
Organize by component type with targeted subdirectories:
```
src/player/
  mod.rs           # Public API (play_session, PlaybackResult)
  state.rs         # PlaybackState, MarkerPosition (shared type), timing state
  input/
    mod.rs         # Input dispatch
    keyboard.rs    # Key event handlers
    mouse.rs       # Mouse event handlers
  playback/
    mod.rs         # Playback controller
    seeking.rs     # Seek operations
    markers.rs     # Marker collection and navigation
  render/
    mod.rs         # Renderer orchestration
    viewport.rs    # Main content viewport
    progress.rs    # Progress bar
    status.rs      # Status bar
    scroll.rs      # Scroll indicators
    help.rs        # Help overlay
    ansi.rs        # Color conversion
```

- Pros:
  - Each file has single, clear responsibility
  - Related code grouped together (e.g., all render components)
  - Easy to add new UI components or input handlers
  - Enables targeted testing of isolated components
- Cons:
  - More files and directories to navigate
  - Some overhead in module boilerplate

## Decision

**Option 3: Component-Based Structure** - This option provides the best balance of:
1. **Separation of concerns** - Each file has a single responsibility
2. **Discoverability** - Related functionality is grouped (all renderers together, all input handlers together)
3. **Testability** - Components can be unit tested in isolation
4. **Extensibility** - New UI elements or input modes can be added without modifying existing code

The `native/` subdirectory from the original proposal is removed since `asciinema.rs` is being deleted entirely, making the nesting unnecessary.

## Consequences

### What becomes easier
- Finding specific functionality (render logic in `render/`, input handling in `input/`)
- Testing individual components (progress bar, ANSI conversion, seeking)
- Adding new features (new render components, new keyboard shortcuts)
- Code review (smaller, focused files)
- Onboarding new contributors

### What becomes harder
- Initial navigation for those unfamiliar with the structure (more files to understand)
- Cross-cutting changes that touch multiple components
- Maintaining module re-exports in `mod.rs` files

### Trade-offs accepted
- More boilerplate in `mod.rs` files for re-exports
- Some functions may need to become `pub(crate)` for cross-module access
- Test files need to import from multiple modules

## Decision History

1. **2025-01-30**: User approved REQUIREMENTS.md defining scope
2. **2025-01-30**: Architect proposed 3 structure options
3. **2025-01-30**: User selected Option 3 (Component-Based) with modification to remove `native/` subdirectory
4. **2025-01-30**: ADR created and accepted
5. **2025-01-30**: Clarifications added to PLAN.md:
   - Resolved open questions: `PlaybackState` as single struct, hybrid input handler approach with `&mut PlaybackState` + `InputResult` enum
   - Clarified `MarkerPosition` location in `state.rs` (shared between render and playback modules)
   - Fixed Stage 8 dependency diagram to show dependency on ALL render stages (2-7)
   - Added note about ~1000 lines of existing inline tests in `native.rs` (Stage 18)
   - Confirmed use of `insta` crate for snapshot testing (Stage 0)
6. **2025-01-30**: Plan simplified from 20 stages to 8 stages:
   - Combined granular render component extractions (Stages 2-8) into single Stage 2
   - Combined playback extractions (Stages 9-10, 15) into single Stage 3
   - Combined input extractions (Stages 12-14) into single Stage 4
   - Combined cleanup stages (Stages 17-19) into Stages 6-7
   - Rationale: Reduces overhead of many small stages while maintaining the same module structure and safety gates (Stage 0 baseline, Stage 8 verification)
7. **2025-01-30**: Snapshot location finalized:
   - Player snapshots stored in `tests/integration/snapshots/player/` (dedicated subdirectory)
   - Test file: `tests/integration/player_snapshot_test.rs`
   - Uses `insta::with_settings!` for custom snapshot path
8. **2025-01-30**: ADR and PLAN signed off - ready for implementation
