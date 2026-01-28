# ADR: Optimize UI Improvements

## Status
Accepted

## Context

The current "transform" feature has several usability issues identified during user testing:

1. **Terminology confusion**: "Transform" is a technical term that doesn't convey what the feature does (silence removal). Users expect clearer, action-oriented terminology like "optimize."

2. **Dialog readability**: The context menu highlight style uses `fg(theme.background)` which resolves to `Color::Reset` (transparent), resulting in white text on a green background that is difficult to read.

3. **Undiscoverable restore**: The 'r' shortcut for restore is not obvious to users. Having both direct shortcuts and context menu access creates confusion about the interaction model.

4. **No visual feedback for optimized files**: After optimizing a file, there's no visual indicator in the file list showing which files have been processed. Users must remember or check manually.

5. **Stale file size**: After optimization, the file size shown in the list doesn't update, creating confusion about whether the operation succeeded.

## Decision

### Comprehensive UI Polish Approach

We will implement a full UI improvement pass that addresses all usability issues while also improving code quality for future maintainability. This includes:

1. **Rename "Transform" to "Optimize" everywhere** - both user-facing strings AND internal identifiers for consistency
2. **Add `highlight_style()` theme method** - reusable style for highlighted/selected items in dialogs
3. **Fix dialog highlighting** - use black text on green background for readability
4. **Remove 'r' direct shortcut** - restore only accessible via context menu
5. **Add [opt] indicator** - styled indicator in file list for optimized files (those with .bak backup)
6. **Add hint/subtitle** - explain what "Optimize" does in the context menu
7. **Auto-refresh file list** - reload file metadata after mutations (optimize, restore, delete)
8. **Update footer shortcuts** - remove 'r' from footer, reflect current shortcuts accurately

### Rationale

- **Internal rename**: Keeping internal code aligned with user-facing terminology reduces cognitive overhead for future developers
- **Reusable highlight_style()**: Prevents the highlighting bug from recurring in future dialogs
- **Context menu only for restore**: Simplifies the mental model - common actions (play, optimize) have shortcuts, rare actions (restore) use menu
- **Auto-refresh**: Users expect the UI to reflect the current state of files after any operation

### Design Details

#### Internal Identifier Renames
```
Mode::TransformResult       -> Mode::OptimizeResult
ContextMenuItem::Transform  -> ContextMenuItem::Optimize
TransformResultState        -> OptimizeResultState
transform_result            -> optimize_result
transform_session()         -> optimize_session()
render_transform_result_modal() -> render_optimize_result_modal()
```

Note: The `Transform` trait in `src/asciicast/transform.rs` and `TransformResult` struct in `transform_ops.rs` will NOT be renamed - they are general-purpose internal abstractions.

#### Theme Addition
```rust
impl Theme {
    /// Style for highlighted/selected items in dialogs and menus.
    /// Uses black text on accent background for readability.
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}
```

#### Context Menu Label Change
```
Before: "Transform (remove silence)"
After:  "Optimize"
        "  Removes silence from recording"  <- hint as subtitle/second line
```

#### File List Indicator
Files with a `.bak` backup will display `[opt]` suffix in accent color:
```
session-2024-01-15.cast [opt]  (claude, 1.5 MB)
```

#### File Metadata Refresh
After optimize/restore operations, reload the file's metadata from disk to update:
- File size
- Modified time (though not currently displayed in list, may be used for sorting)

## Consequences

### Positive
- Clearer, more intuitive terminology for users
- Readable dialog highlighting
- Consistent interaction model (shortcuts for common, menu for rare)
- Visual feedback for optimization status at a glance
- Accurate file sizes after operations
- Reusable theme pattern for future dialogs
- Internal code aligns with user-facing terms

### Negative
- More changes = more testing required
- Snapshot tests will need updates
- May affect any external documentation referencing "transform"

### Risks
- **Low**: All changes are in the TUI layer, no changes to core transform logic
- **Medium**: Snapshot test updates may be tedious but straightforward
- **Mitigation**: Break into small stages, verify each stage independently

## Testing Strategy

1. **Unit tests**: Update existing tests for renamed identifiers
2. **Snapshot tests**: Update TUI snapshots for:
   - Context menu appearance
   - Help modal (remove 'r' shortcut)
   - Optimize result modal
   - File list with [opt] indicator
3. **Manual testing**: Verify dialog readability, file refresh after optimize/restore
4. **Regression**: Ensure core transform functionality unchanged
