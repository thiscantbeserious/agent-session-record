# Execution Plan: Optimize UI Improvements

## Overview
Comprehensive UI polish for the transform/optimize feature. Broken into 10 small, testable stages.

**Branch:** `feature/optimize-ui-improvements`

---

## Stage 1: Add `highlight_style()` to Theme

### Objective
Add a reusable theme method for highlighted/selected items in dialogs.

### Files to Modify
- `src/tui/theme.rs`

### Implementation
1. Add `highlight_style()` method to the `Theme` impl block
2. Returns black text on accent background with bold modifier
3. Add unit test for the new method

### Code Change
```rust
// Add to Theme impl (after success_style):

/// Style for highlighted/selected items in dialogs and menus.
/// Uses black text on accent background for readability.
pub fn highlight_style(&self) -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(self.accent)
        .add_modifier(Modifier::BOLD)
}
```

### Testing
```bash
cargo test -p agent-session-recorder theme
```

### Verification
- New method exists and compiles
- Unit test passes

---

## Stage 2: Fix Context Menu Highlight Styling

### Objective
Use the new `highlight_style()` for context menu selected items instead of broken `fg(theme.background)`.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. In `render_context_menu_modal()`, replace the manual style construction with `theme.highlight_style()`
2. This fixes the readability issue (black text on green)

### Code Change
```rust
// In render_context_menu_modal(), replace:
let style = if is_selected {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else if is_disabled {
    // ...
};

// With:
let style = if is_selected {
    theme.highlight_style()
} else if is_disabled {
    // ...
};
```

### Testing
```bash
cargo test -p agent-session-recorder list_app
cargo insta test
```

### Verification
- Context menu highlight is now black text on green
- Manual testing: open context menu, verify readability

---

## Stage 3: Rename Mode::TransformResult to Mode::OptimizeResult

### Objective
Begin internal identifier rename - start with the Mode enum variant.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. Rename `Mode::TransformResult` to `Mode::OptimizeResult`
2. Update all references in the file (match arms, comparisons)
3. Update test names if any reference the old name

### Code Change
```rust
// In Mode enum:
/// Optimize result mode - showing optimization results or error
OptimizeResult,

// Update all match arms and references
```

### Testing
```bash
cargo test -p agent-session-recorder list_app
```

### Verification
- All tests pass
- No compiler errors

---

## Stage 4: Rename TransformResultState and Related Identifiers

### Objective
Complete internal identifier rename for result state.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. Rename `TransformResultState` struct to `OptimizeResultState`
2. Rename `transform_result` field to `optimize_result`
3. Rename `handle_transform_result_key()` to `handle_optimize_result_key()`
4. Rename `render_transform_result_modal()` to `render_optimize_result_modal()`
5. Update all references

### Code Change
```rust
/// Holds the result of an optimize operation for display in modal.
#[derive(Debug, Clone)]
pub struct OptimizeResultState {
    /// The filename that was optimized
    pub filename: String,
    /// The result (Ok with data or Err with message)
    pub result: Result<TransformResult, String>,  // Keep TransformResult from asciicast module
}

// Rename field:
optimize_result: Option<OptimizeResultState>,

// Rename methods:
fn handle_optimize_result_key(...) { ... }
pub fn render_optimize_result_modal(...) { ... }
```

### Testing
```bash
cargo test -p agent-session-recorder list_app
cargo insta test
```

### Verification
- All tests pass
- Snapshot tests may need updates

---

## Stage 5: Rename ContextMenuItem::Transform and transform_session()

### Objective
Complete context menu and action method rename.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. Rename `ContextMenuItem::Transform` to `ContextMenuItem::Optimize`
2. Rename `transform_session()` to `optimize_session()`
3. Update `label()` method: "Transform (remove silence)" -> "Optimize"
4. Update `shortcut()` comment if needed (still 't')
5. Update all references in match arms

### Code Change
```rust
pub enum ContextMenuItem {
    Play,
    Optimize,  // was Transform
    Restore,
    Delete,
    AddMarker,
}

impl ContextMenuItem {
    pub fn label(&self) -> &'static str {
        match self {
            ContextMenuItem::Optimize => "Optimize",
            // ...
        }
    }
}

// Rename method:
fn optimize_session(&mut self) -> Result<()> { ... }
```

### Testing
```bash
cargo test -p agent-session-recorder list_app
```

### Verification
- Tests pass (update test assertions for new enum variant name)

---

## Stage 6: Update UI Strings and Modal Titles

### Objective
Update all user-facing strings from "transform" to "optimize."

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. Footer text: "t: transform" -> "t: optimize"
2. Help modal: "Transform (remove silence)" -> "Optimize (removes silence)"
3. Result modal title: "Transform Complete" -> "Optimization Complete"
4. Result modal title: "Transform Failed" -> "Optimization Failed"
5. Status messages (if any)

### Code Change
```rust
// Footer (Mode::Normal):
"...| t: optimize | ..."

// Help modal:
Line::from(vec![
    Span::styled("  t", Style::default().fg(theme.accent)),
    Span::raw("           Optimize (removes silence)"),
]),

// Result modal titles:
let title = " Optimization Complete ";
let title = " Optimization Failed ";
```

### Testing
```bash
cargo insta test
```

### Verification
- Update all affected snapshots
- Manual verification of UI text

---

## Stage 7: Remove 'r' Direct Shortcut

### Objective
Remove the direct 'r' keyboard shortcut for restore; keep restore only in context menu.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. Remove `KeyCode::Char('r')` handler from `handle_normal_key()`
2. Remove "r: restore" from footer text in `Mode::Normal`
3. Remove 'r' shortcut line from help modal
4. Keep restore functionality in `execute_context_menu_action()`

### Code Change
```rust
// In handle_normal_key(), DELETE:
KeyCode::Char('r') => {
    // ... entire block
}

// In footer (Mode::Normal), change:
"↑↓: navigate | Enter: menu | p: play | t: optimize | d: delete | ?: help | q: quit"
// (removed "| r: restore")

// In render_help_modal(), DELETE:
Line::from(vec![
    Span::styled("  r", Style::default().fg(theme.accent)),
    Span::raw("           Restore from backup"),
]),
```

### Testing
```bash
cargo test -p agent-session-recorder list_app
cargo insta test
```

### Verification
- Pressing 'r' in normal mode does nothing
- Restore still works via context menu
- Footer and help modal updated

---

## Stage 8: Add Hint/Subtitle to Context Menu

### Objective
Add a description line under "Optimize" in the context menu explaining what it does.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. In `render_context_menu_modal()`, add a subtitle line after "Optimize"
2. Style as secondary text
3. Only show for Optimize item (not all items)

### Code Change
```rust
// In render_context_menu_modal(), modify the loop:
for (idx, item) in ContextMenuItem::ALL.iter().enumerate() {
    // ... existing code to build label and style ...

    lines.push(Line::from(Span::styled(
        format!("{}{}", prefix, label),
        style,
    )));

    // Add hint for Optimize
    if matches!(item, ContextMenuItem::Optimize) {
        lines.push(Line::from(Span::styled(
            "       Removes silence from recording",
            Style::default().fg(theme.text_secondary),
        )));
    }
}
```

### Testing
```bash
cargo insta test
```

### Verification
- Context menu shows hint under Optimize
- Hint is dimmed/secondary color

---

## Stage 9: Add [opt] Indicator for Optimized Files

### Objective
Display `[opt]` indicator in file list for files that have a .bak backup.

### Files to Modify
- `src/tui/widgets/file_explorer.rs`
- `src/tui/list_app.rs` (to pass backup status per file)

### Implementation

Option A (Simpler): Check backup in FileExplorerWidget render
1. In `FileExplorerWidget::render()`, for each visible item, check `has_backup()`
2. Add `[opt]` span with accent color if backup exists

Option B (Better performance): Pre-compute and pass backup status
1. Add `has_backup: bool` field to `FileItem` or compute during iteration
2. Pass backup status through visible_items iteration

We'll use Option A for simplicity since backup check is just a file existence check.

### Code Change (file_explorer.rs)
```rust
// In FileExplorerWidget render(), modify item_data collection:
use crate::asciicast::has_backup;

let item_data: Vec<(String, String, String, bool, bool)> = self
    .explorer
    .visible_items()
    .map(|(_, item, is_checked)| {
        let has_bak = has_backup(std::path::Path::new(&item.path));
        (
            item.name.clone(),
            item.agent.clone(),
            format_size(item.size),
            is_checked,
            has_bak,  // new field
        )
    })
    .collect();

// In the items iterator, add [opt] indicator:
let items: Vec<ListItem> = item_data
    .iter()
    .map(|(name, agent, size_str, is_checked, has_bak)| {
        let mut spans = vec![];
        if show_checkboxes { /* ... */ }
        spans.push(Span::styled(name.as_str(), theme.text_style()));

        // Add [opt] indicator if backup exists
        if *has_bak {
            spans.push(Span::styled(" [opt]", theme.accent_style()));
        }

        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("({}, {})", agent, size_str),
            theme.text_secondary_style(),
        ));
        ListItem::new(Line::from(spans))
    })
    .collect();
```

### Testing
```bash
cargo test -p agent-session-recorder file_explorer
cargo insta test
```

### Verification
- Files with .bak show [opt] in green
- Files without backup don't show indicator
- Indicator updates after optimize/restore

---

## Stage 10: Auto-Refresh File Metadata After Mutations

### Objective
Update file size (and other metadata) in the list after optimize/restore operations.

### Files to Modify
- `src/tui/widgets/file_explorer.rs`
- `src/tui/list_app.rs`

### Implementation
1. Add `update_item_metadata()` method to `FileExplorer` that reloads a file's size from disk
2. Call this method after `optimize_session()` and `restore_session()` succeed
3. The method finds the item by path and updates its size field

### Code Change (file_explorer.rs)
```rust
impl FileExplorer {
    /// Update metadata for an item by reloading from disk.
    /// Returns true if item was found and updated.
    pub fn update_item_metadata(&mut self, path: &str) -> bool {
        if let Some(idx) = self.items.iter().position(|item| item.path == path) {
            // Reload metadata from disk
            if let Ok(metadata) = std::fs::metadata(path) {
                self.items[idx].size = metadata.len();
                // Could also update modified time if needed
                return true;
            }
        }
        false
    }
}
```

### Code Change (list_app.rs)
```rust
// In optimize_session(), after successful transform:
Ok(result) => {
    self.preview_cache.invalidate(path);
    // Refresh file metadata in explorer
    self.explorer.update_item_metadata(&item.path);
    Ok(result)
}

// In restore_session(), after successful restore:
Ok(()) => {
    self.preview_cache.invalidate(path);
    // Refresh file metadata in explorer
    self.explorer.update_item_metadata(&item.path);
    self.status_message = Some(format!("Restored from backup: {}", name));
}
```

### Testing
```bash
cargo test -p agent-session-recorder file_explorer
cargo test -p agent-session-recorder list_app
```

### Verification
- File size updates after optimize (should decrease)
- File size updates after restore (may change)
- [opt] indicator updates correctly

---

## Completion Criteria

All stages complete when:

1. `cargo build` succeeds
2. `cargo test` passes
3. `cargo clippy` reports no warnings
4. `cargo fmt --check` passes
5. All snapshot tests updated and passing
6. Manual verification:
   - Context menu shows "Optimize" with readable highlight
   - Hint appears under Optimize in menu
   - 'r' shortcut removed, restore only via menu
   - [opt] indicator appears for optimized files
   - File size updates after operations
   - Footer shows correct shortcuts
   - Help modal updated

---

## Stage Dependency Graph

```
Stage 1 (theme) ─────┐
                     v
Stage 2 (fix highlight) ──> Stage 6 (UI strings)
                     │
Stage 3 (Mode rename) ──> Stage 4 (struct rename) ──> Stage 5 (method rename)
                                                              │
                                                              v
                     Stage 7 (remove 'r') ──> Stage 8 (hint) ──> Stage 9 ([opt])
                                                                        │
                                                                        v
                                                              Stage 10 (refresh)
```

Stages 1-2 can run in parallel with Stages 3-5.
Stages 6-10 should run sequentially.
