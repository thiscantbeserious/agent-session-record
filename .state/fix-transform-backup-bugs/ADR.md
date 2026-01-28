# ADR: Fix Transform and Backup Bugs from PR #65 Review

## Status
Accepted

## Context

Five bugs were identified during adversarial review of PR #65 (context menu and transform integration). These bugs affect:
- Data integrity (temp file leaks, non-atomic operations)
- UX (selectable disabled options)
- CI consistency (version mismatch)

The existing codebase already uses the temp+rename pattern for `apply_transforms()`, establishing a precedent for atomic file operations.

## Decision

### Approach: Targeted Fixes with Existing Patterns

Since these are well-defined bugfixes (not new features), the approach is straightforward:
- Apply the established temp+rename pattern to `restore_from_backup()`
- Add cleanup logic for failure paths in existing operations
- Add guard logic for disabled context menu items
- Align CI version with Cargo.toml

**Rationale:** The codebase already demonstrates the correct patterns (temp+rename in `apply_transforms`). These bugs are deviations from those patterns that need alignment.

### Fix Details

#### Bug 1: Temp file cleanup on rename failure
**Location:** `src/asciicast/transform_ops.rs:108-115`
**Fix:** Wrap the rename in a match and clean up temp file on error before propagating.

```rust
// Current (buggy):
fs::rename(&temp_path, path)
    .with_context(|| ...)?;

// Fixed:
if let Err(e) = fs::rename(&temp_path, path) {
    let _ = fs::remove_file(&temp_path); // Best-effort cleanup
    return Err(e).with_context(|| ...);
}
```

#### Bug 2: Atomic restore operation
**Location:** `src/asciicast/transform_ops.rs:141-152`
**Fix:** Apply the same temp+rename pattern used by `apply_transforms()`.

```rust
// Current (non-atomic):
fs::copy(&backup, path)?;

// Fixed:
let temp_path = path.with_extension("cast.tmp");
fs::copy(&backup, &temp_path)?;
if let Err(e) = fs::rename(&temp_path, path) {
    let _ = fs::remove_file(&temp_path);
    return Err(e).with_context(|| ...);
}
```

#### Bug 3: Delete removes backup file
**Location:** `src/tui/list_app.rs:562-576`
**Fix:** After successful main file deletion, attempt to delete `.cast.bak` if it exists.

```rust
// After successful remove_file(&path):
let backup = backup_path_for(&path);
if backup.exists() {
    let _ = std::fs::remove_file(&backup); // Best-effort
    // Optionally update status message
}
```

#### Bug 4: Disabled Restore option not selectable
**Location:** `src/tui/list_app.rs:523-539`
**Fix:** Check if Restore is disabled before executing the action.

```rust
fn execute_context_menu_action(&mut self) -> Result<()> {
    let action = ContextMenuItem::ALL[self.context_menu_idx];

    // Guard: don't execute disabled Restore
    if matches!(action, ContextMenuItem::Restore) {
        if let Some(item) = self.explorer.selected_item() {
            if !has_backup(Path::new(&item.path)) {
                self.status_message = Some("No backup exists".into());
                self.mode = Mode::Normal;
                return Ok(());
            }
        }
    }

    self.mode = Mode::Normal;
    // ... rest of match
}
```

#### Bug 5: cargo-insta version alignment
**Location:** `.github/workflows/ci.yml:199`
**Fix:** Change `1.43.1` to `1.46.1`.

## Consequences

### Positive
- Temp files will be cleaned up on all failure paths
- Restore operation becomes crash-safe (atomic)
- Deleting recordings cleans up all associated files
- Context menu UX matches visual state
- CI/local development consistency

### Negative
- None significant; these are correctness fixes

### Risks
- Low risk; all changes are localized and testable
- Atomic operations may behave differently on network filesystems (rename semantics vary), but this matches existing `apply_transforms()` behavior

## Testing Strategy

Each bug fix should have corresponding test coverage:
1. **Bug 1:** Test that temp file doesn't exist after failed rename (mock or use permissions)
2. **Bug 2:** Test restore atomicity (verify backup unchanged after restore, test idempotency)
3. **Bug 3:** Test delete removes both `.cast` and `.cast.bak`
4. **Bug 4:** Test that disabled Restore shows message but doesn't call restore_from_backup
5. **Bug 5:** Run snapshot tests after CI update
