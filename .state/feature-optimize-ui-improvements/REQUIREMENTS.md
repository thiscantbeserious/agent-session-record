# Requirements: Optimize UI Improvements

## Problem Statement
The current "transform" feature has usability issues: the terminology is unclear ("transform" doesn't convey what happens), the dialog has readability problems with highlighted text (white on green), the 'r' shortcut for restore is undiscoverable, and there's no visual indicator for which files have been optimized.

## User Stories
- As a user, I want the feature called "optimize" instead of "transform" so that the terminology is clearer
- As a user, I want a hint explaining what optimization does so that I understand the operation
- As a user, I want to read highlighted text in dialogs so that I can see which option is selected
- As a user, I want restore to only be accessible via the context menu to simplify the shortcut model
- As a user, I want to see which files have been optimized so that I can track status at a glance
- As a user, I want the file size to update after optimization so the list reflects current state

## Acceptance Criteria

### Terminology Change
1. [ ] All UI references to "transform" are renamed to "optimize"
2. [ ] Menu items, labels, and prompts use "optimize" terminology
3. [ ] A hint/description explains what optimization does (e.g., "removes silence")

### Dialog Highlighting Fix
4. [ ] Highlighted line text in dialogs uses black color (not white)
5. [ ] Highlighted text is readable against green highlight background

### Restore Access Change
6. [ ] Direct 'r' keyboard shortcut for restore is removed
7. [ ] Restore action is only accessible through the context menu
8. [ ] Context menu still contains the restore option

### Optimized File Indicator
9. [ ] Files with a `.bak` backup file display an indicator (e.g., [o]) in the file list
10. [ ] The indicator is removed when the file is restored (backup deleted)
11. [ ] The indicator is visible and distinguishable from the filename

### File Size Update
12. [ ] File size in the list updates after optimization to reflect actual current size

## Out of Scope
- Changes to the actual optimization algorithm (silence removal logic)
- New optimization options or settings
- Rename functionality
- Undo/redo beyond single restore from backup

## Open Questions
- None - requirements confirmed
