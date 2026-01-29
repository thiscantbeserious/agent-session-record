# Requirements: Fix Player Scroll Region Bug

## Sign-off

- [x] Requirements reviewed by Product Owner
- [ ] Requirements approved by user
- [ ] Implementation complete
- [ ] Validation passed

## Problem Statement

Our native player renders terminal output differently from asciinema and standard terminal emulators (pyte). Content appears at wrong line positions because our VT emulator ignores scroll region commands.

**User Impact:** When users play back recordings of TUI applications (vim, tmux, codex CLI, htop, etc.), the displayed content appears at incorrect vertical positions. This makes recordings appear broken or corrupted, degrading trust in the tool and making playback unusable for debugging or review purposes.

## Investigation Summary

### Visual Comparison (pyte vs our player at 10000 events)

**Pyte output:**
```
 1: |  filename and filename alone, using glob::Pattern, with error handlin|
 2: |  introduce dialoguer for interactive UI enhancements and update CLI p|
 4: |  Designing interactive session list and cleanup UI                   |
...content at lines 1-55...
```

**Our player output:**
```
45: |• Model changed to gpt-5.2-codex medium|
47: |• Explored|
48: |  └ Read storage.rs|
50: |• Designing interactive list and cleanup flows (4m 02s • esc to interrupt)|
...content at lines 45-55...
```

Same content, but at **different line positions**. Our player shows content 44 lines lower.

### Root Cause

The test file contains **2,438 scroll region commands** in the first 10000 events:
- `CSI r` (DECSTBM - Set Top and Bottom Margins): ~2438 occurrences
- `CSI S` (Scroll Up): ~106 occurrences
- `CSI T` / `ESC M` (Scroll Down / Reverse Index): ~116 occurrences

Our `TerminalBuffer` in `src/player/terminal.rs` has:
```rust
match action {
    'A' => { /* cursor up */ }
    'B' => { /* cursor down */ }
    // ... other handlers ...
    _ => {} // <-- SILENTLY IGNORES 'r', 'S', 'T'
}
```

The `_ => {}` catch-all silently ignores:
- `'r'` - Set scroll region (DECSTBM)
- `'S'` - Scroll up n lines
- `'T'` - Scroll down n lines

### Impact

- Playback looks wrong for recordings with TUI apps (vim, tmux, codex, etc.)
- Content appears at wrong vertical positions
- Users may think files are corrupted when they're actually fine
- Seeking/jumping in playback produces wrong visual state

## Acceptance Criteria

### User-Facing Requirements (CRITICAL)
1. [ ] **Visual parity with standard terminals**: Playing the test file shows content at the same line positions as pyte/asciinema
2. [ ] **No regression for simple recordings**: Recordings without scroll regions continue to play correctly
3. [ ] **TUI app compatibility**: Recordings of vim, tmux, codex CLI render correctly

### Scroll Region Support (HIGH - Implementation Details)
4. [ ] Add `scroll_top` and `scroll_bottom` fields to `TerminalBuffer`
5. [ ] Implement `CSI r` handler (DECSTBM - Set Top and Bottom Margins)
   - `CSI r` with no params: reset to full screen (1 to height)
   - `CSI top;bottom r`: set scroll region to lines top-bottom (1-indexed)
6. [ ] Update existing scroll operations to respect scroll region bounds

### Scroll Up/Down Commands (HIGH - Implementation Details)
7. [ ] Implement `CSI S` handler (Scroll Up)
   - Scroll content up n lines within scroll region
   - Bottom lines become blank
8. [ ] Implement `CSI T` handler (Scroll Down)
   - Scroll content down n lines within scroll region
   - Top lines become blank
9. [ ] Update `ESC M` (Reverse Index) to respect scroll region

### Verification & Testing (HIGH)
10. [ ] Visual comparison test passes at multiple checkpoints (1000, 5000, 10000 events)
11. [ ] All existing terminal tests continue to pass (`cargo test`)
12. [ ] Manual verification: play test file and confirm no visual artifacts

### Edge Cases (MEDIUM)
13. [ ] Scroll region reset on terminal resize
14. [ ] Invalid scroll region params handled gracefully (ignored or clamped)
15. [ ] Cursor constrained appropriately when inside/outside scroll region

## Test File

Primary test file for verification:
```
/Users/simon.sanladerer/recorded_agent_sessions/codex/agr_codex_failed_interactively.cast
```
- 61,338 events
- 106x70 terminal
- Heavy use of scroll regions (codex TUI)

## Out of Scope

- Other missing VT sequences (unless discovered during implementation)
- Performance optimization
- Alternate screen buffer handling

## Technical Notes

### Example scroll region sequence from test file:
```
\x1b[?2026h\x1b[1;69r\x1b[4S\x1b[r
```
Breakdown:
- `\x1b[?2026h` - DEC private mode (ignored, fine)
- `\x1b[1;69r` - Set scroll region lines 1-69
- `\x1b[4S` - Scroll up 4 lines
- `\x1b[r` - Reset scroll region to full screen

### Reference implementations:
- pyte (Python): https://github.com/selectel/pyte
- vte (Rust crate we use): Already parses these, we just don't handle them

## Verification Method

To verify this fix works correctly:

1. **Automated test**: Run the visual comparison test that compares our output against pyte at event checkpoints
   ```bash
   cargo test --test visual_comparison
   ```

2. **Manual verification**: Play the test file and visually confirm:
   ```bash
   cargo run -- play /Users/simon.sanladerer/recorded_agent_sessions/codex/agr_codex_failed_interactively.cast
   ```
   - Content should appear at correct line positions throughout playback
   - Seek/jump should produce correct visual state
   - No visual "tearing" or content appearing in wrong regions

3. **Regression check**: Ensure all existing tests pass
   ```bash
   cargo test
   ```

## Definition of Done

- [ ] All acceptance criteria marked as complete
- [ ] Code reviewed by Reviewer role
- [ ] Tests pass in CI
- [ ] Product Owner validates user-facing requirements are met

## Context

- Branch: `fix/player-scroll-region-bug`
- Visual comparison test added in initial commit
- pyte installed for reference comparison: `pip3 install pyte`
