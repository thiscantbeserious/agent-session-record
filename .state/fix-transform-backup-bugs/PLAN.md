# Execution Plan: Fix Transform and Backup Bugs

## Overview
Five targeted bugfixes from PR #65 review. Each stage fixes one bug and includes relevant tests.

---

## Stage 1: Temp File Cleanup on Rename Failure

### Objective
Ensure temp files are cleaned up when `fs::rename()` fails in `apply_transforms()`.

### Files to Modify
- `src/asciicast/transform_ops.rs`

### Implementation
1. Change lines 114-115 from direct `?` propagation to explicit error handling
2. On rename failure, attempt to remove the temp file before returning error
3. Preserve original error context in the returned error

### Code Change
```rust
// Replace:
fs::rename(&temp_path, path)
    .with_context(|| format!("Failed to replace original file: {}", path.display()))?;

// With:
if let Err(e) = fs::rename(&temp_path, path) {
    // Clean up temp file on failure (best-effort, ignore cleanup errors)
    let _ = fs::remove_file(&temp_path);
    return Err(e).with_context(|| format!("Failed to replace original file: {}", path.display()));
}
```

### Testing
- Existing tests must continue to pass
- Consider adding a test that verifies no `.cast.tmp` files remain after operation (both success and failure paths)

### Verification
```bash
cargo test -p agent-session-recorder transform_ops
```

---

## Stage 2: Atomic Restore Operation

### Objective
Make `restore_from_backup()` use the same atomic temp+rename pattern as `apply_transforms()`.

### Files to Modify
- `src/asciicast/transform_ops.rs`

### Implementation
1. Replace direct `fs::copy(&backup, path)` with atomic pattern:
   - Copy backup to temp file (`.cast.tmp`)
   - Rename temp file to target
   - Clean up temp on rename failure

### Code Change
```rust
pub fn restore_from_backup(path: &Path) -> Result<()> {
    let backup = backup_path_for(path);

    if !backup.exists() {
        anyhow::bail!("No backup exists for: {}", path.display());
    }

    // Use atomic temp+rename pattern for crash safety
    let temp_path = path.with_extension("cast.tmp");

    fs::copy(&backup, &temp_path)
        .with_context(|| format!("Failed to copy backup to temp file: {}", backup.display()))?;

    if let Err(e) = fs::rename(&temp_path, path) {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        return Err(e).with_context(|| format!("Failed to restore from backup: {}", path.display()));
    }

    Ok(())
}
```

### Testing
- Existing `restore_from_backup` tests must pass
- Add test verifying backup file is unchanged after restore
- Verify idempotency: multiple restores produce same result

### Verification
```bash
cargo test -p agent-session-recorder restore_from_backup
```

---

## Stage 3: Delete Removes Backup File

### Objective
When deleting a `.cast` file, also delete the corresponding `.cast.bak` if it exists.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. In `delete_session()`, after successful main file deletion
2. Check if backup exists using `backup_path_for()`
3. Attempt to delete backup (best-effort, don't fail if backup deletion fails)
4. Update status message to indicate if backup was also deleted

### Code Change
```rust
fn delete_session(&mut self) -> Result<()> {
    if let Some(item) = self.explorer.selected_item() {
        let path = item.path.clone();
        let name = item.name.clone();

        // Delete the file
        if let Err(e) = std::fs::remove_file(&path) {
            self.status_message = Some(format!("Failed to delete: {}", e));
        } else {
            // Also delete backup if it exists
            let backup = backup_path_for(std::path::Path::new(&path));
            let backup_deleted = if backup.exists() {
                std::fs::remove_file(&backup).is_ok()
            } else {
                false
            };

            // Remove from explorer to keep UI in sync
            self.explorer.remove_item(&path);

            // Update status message
            self.status_message = Some(if backup_deleted {
                format!("Deleted: {} (and backup)", name)
            } else {
                format!("Deleted: {}", name)
            });
        }
    }
    Ok(())
}
```

### Testing
- Manual testing: delete a file with backup, verify both are removed
- Manual testing: delete a file without backup, verify clean deletion

### Verification
```bash
cargo test -p agent-session-recorder list_app
cargo clippy
```

---

## Stage 4: Disabled Restore Option Not Selectable

### Objective
Prevent executing `restore_session()` when Restore is disabled (no backup exists) via context menu.

### Files to Modify
- `src/tui/list_app.rs`

### Implementation
1. In `execute_context_menu_action()`, before executing Restore
2. Check if backup exists
3. If no backup, show status message and return early without calling `restore_session()`

### Code Change
```rust
fn execute_context_menu_action(&mut self) -> Result<()> {
    let action = ContextMenuItem::ALL[self.context_menu_idx];

    // Guard: check if Restore is disabled (no backup)
    if matches!(action, ContextMenuItem::Restore) {
        if let Some(item) = self.explorer.selected_item() {
            let path = std::path::Path::new(&item.path);
            if !has_backup(path) {
                self.mode = Mode::Normal;
                self.status_message = Some("No backup exists for this file".to_string());
                return Ok(());
            }
        }
    }

    self.mode = Mode::Normal; // Close menu first

    match action {
        // ... existing match arms
    }
    Ok(())
}
```

### Testing
- Manual testing: open context menu on file without backup, select Restore, verify message shown
- Verify `r` shortcut in normal mode still shows appropriate message

### Verification
```bash
cargo test -p agent-session-recorder list_app
cargo clippy
```

---

## Stage 5: Align cargo-insta Version in CI

### Objective
Ensure CI uses the same cargo-insta version as Cargo.toml.

### Files to Modify
- `.github/workflows/ci.yml`

### Implementation
1. Change line 199 from `1.43.1` to `1.46.1`

### Code Change
```yaml
# Replace:
run: cargo install cargo-insta --version 1.43.1

# With:
run: cargo install cargo-insta --version 1.46.1
```

### Testing
- Run snapshot tests locally to verify they pass

### Verification
```bash
cargo insta test
```

---

## Completion Criteria

All stages complete when:
1. `cargo test` passes
2. `cargo clippy` reports no warnings
3. `cargo fmt --check` passes
4. All five bugs are fixed as specified in REQUIREMENTS.md
