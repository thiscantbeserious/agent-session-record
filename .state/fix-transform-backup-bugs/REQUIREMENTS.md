# Requirements: Fix Transform and Backup Bugs from PR #65 Review

## Problem Statement

Five bugs were identified during adversarial review of PR #65 (context menu and transform integration). These bugs range from data integrity issues (temp file leaks, non-atomic operations) to UX problems (selectable disabled options) and CI inconsistencies. All must be fixed to ensure reliable transform operations and proper user experience.

## User Stories

- As a user, I want temp files cleaned up after failures so that my disk doesn't fill with orphaned `.cast.tmp` files
- As a user, I want restore operations to be atomic so that a crash during restore doesn't corrupt both my file and backup
- As a user, I want delete to remove all associated files so that backup files don't accumulate and waste disk space
- As a user, I want disabled menu options to be truly non-selectable so that I don't get confusing error messages
- As a maintainer, I want CI and local development to use the same cargo-insta version so that snapshot tests behave consistently

## Acceptance Criteria

### Bug 1: Temp file cleanup on rename failure (HIGH)
**Location:** `src/asciicast/transform_ops.rs:108-115`

1. When `fs::rename()` fails after writing to `.cast.tmp`, the temp file MUST be deleted
2. The cleanup MUST happen regardless of why the rename failed (permissions, disk full, etc.)
3. Original error context MUST be preserved in the returned error message
4. Existing unit tests MUST continue to pass

### Bug 2: Atomic restore operation (HIGH)
**Location:** `src/asciicast/transform_ops.rs:141-152`

1. `restore_from_backup()` MUST use atomic temp+rename pattern (write to temp file, then rename)
2. If rename fails, temp file MUST be cleaned up
3. A crash during restore MUST NOT corrupt the original file
4. A crash during restore MUST NOT corrupt the backup file
5. Restore operation MUST remain idempotent (can be retried on failure)

### Bug 3: Delete removes backup file (HIGH)
**Location:** `src/tui/list_app.rs:562-576`

1. When deleting a `.cast` file, the corresponding `.cast.bak` file MUST also be deleted if it exists
2. Failure to delete backup MUST NOT prevent deletion of the main file
3. Status message SHOULD indicate if backup was also deleted
4. If only backup deletion fails, a warning SHOULD be shown but operation succeeds

### Bug 4: Disabled Restore option not selectable (MEDIUM)
**Location:** `src/tui/list_app.rs` (context menu handling)

1. When Restore option is disabled (no backup exists), pressing Enter on it MUST NOT execute `restore_session()`
2. Arrow keys MUST skip over disabled items OR Enter on disabled item MUST be a no-op with visual feedback
3. The current visual styling (grayed out) MUST remain
4. Shortcut key `r` in normal mode SHOULD behave consistently (either skip or show "no backup" message)

### Bug 5: cargo-insta version alignment (LOW)
**Location:** `Cargo.toml:55` and `.github/workflows/ci.yml:199`

1. CI workflow MUST use same cargo-insta version as Cargo.toml
2. Version in CI: `1.43.1` -> `1.46.1` (to match Cargo.toml)
3. All snapshot tests MUST pass after version update

## Out of Scope

- Adding new transform operations
- Changing backup file naming conventions (`.bak` suffix)
- Adding compression or encryption to backups
- Changing context menu keyboard navigation paradigm
- Upgrading other CI dependencies

## Constraints

- Must maintain backward compatibility with existing `.cast` and `.cast.bak` files
- Atomic operations should use temp+rename pattern consistent with existing `apply_transforms()` code
- No new dependencies should be added for file operations (use `std::fs`)
- Tests must remain deterministic and not rely on timing

## Technical Notes

The temp+rename pattern for atomicity:
1. Write to `<file>.tmp`
2. Call `fs::rename(temp, target)` - atomic on most filesystems
3. On rename failure, clean up temp file
4. This prevents partial writes from corrupting the target

For context menu disabled items, two valid approaches:
- Skip disabled items during navigation (up/down arrows)
- Allow selection but make Enter a no-op with feedback

Either approach is acceptable as long as `restore_session()` is never called when no backup exists via the context menu path.
