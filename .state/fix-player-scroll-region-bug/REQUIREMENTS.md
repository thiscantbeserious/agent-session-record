# Requirements: Fix Player Scroll Region Bug

## Problem Statement

Our native player renders terminal output differently from asciinema and standard terminal emulators (pyte). Content appears at wrong line positions because our VT emulator ignores scroll region commands.

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

### Scroll Region Support (HIGH)
1. [ ] Add `scroll_top` and `scroll_bottom` fields to `TerminalBuffer`
2. [ ] Implement `CSI r` handler (DECSTBM - Set Top and Bottom Margins)
   - `CSI r` with no params: reset to full screen (1 to height)
   - `CSI top;bottom r`: set scroll region to lines top-bottom (1-indexed)
3. [ ] Update existing scroll operations to respect scroll region bounds

### Scroll Up/Down Commands (HIGH)
4. [ ] Implement `CSI S` handler (Scroll Up)
   - Scroll content up n lines within scroll region
   - Bottom lines become blank
5. [ ] Implement `CSI T` handler (Scroll Down)
   - Scroll content down n lines within scroll region
   - Top lines become blank
6. [ ] Update `ESC M` (Reverse Index) to respect scroll region

### Visual Verification (HIGH)
7. [ ] Visual comparison test passes (our output matches pyte at checkpoints)
8. [ ] Existing terminal tests continue to pass

### Edge Cases (MEDIUM)
9. [ ] Scroll region reset on terminal resize
10. [ ] Invalid scroll region params handled gracefully (ignored or clamped)
11. [ ] Cursor clamped to scroll region when appropriate

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

## Context

- Branch: `fix/player-scroll-region-bug`
- Visual comparison test added in initial commit
- pyte installed for reference comparison: `pip3 install pyte`
