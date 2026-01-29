# ADR: Fix Player Scroll Region Bug

## Status
Approved

## Architect Review Notes

**Reviewed:** 2026-01-29

**Assessment:** The design is technically sound and aligns with the codebase architecture.

**Implementation Notes:**
1. The `scroll_up`/`scroll_down` methods should be implemented on `TerminalPerformer` to match the existing pattern (all buffer manipulation methods like `line_feed`, `delete_lines`, `insert_lines` are on the performer)
2. The scroll region fields (`scroll_top`, `scroll_bottom`) should be stored in `TerminalBuffer` and passed to `TerminalPerformer` via its constructor
3. The `line_feed` method (line 247) should also be updated to respect scroll regions for full correctness - when cursor is at `scroll_bottom`, scroll within the region rather than the full screen

**Known Limitations (acceptable for this fix):**
- DECOM (origin mode) is not implemented - cursor positioning is always absolute
- Some edge cases may differ slightly from xterm/vt100 behavior

**Technical Risks (low):**
- Performance impact of scroll operations is negligible for typical recordings
- Edge cases in scroll region handling are well-defined by the DECSTBM spec

## Context

Our native player's `TerminalBuffer` (in `src/player/terminal.rs`) uses the `vte` crate to parse ANSI escape sequences. While `vte` correctly parses scroll-related sequences, our `csi_dispatch` handler silently ignores them with a `_ => {}` catch-all.

This causes visual output to differ from standard terminal emulators when playing recordings that use scroll regions (common in TUI apps like vim, tmux, codex CLI, etc.).

### Current State

```rust
// src/player/terminal.rs, csi_dispatch()
match action {
    'A' => { /* cursor up */ }
    'B' => { /* cursor down */ }
    // ... handlers for H, J, K, L, M, P, @, X, s, u, G, d, m ...
    _ => {} // PROBLEM: ignores 'r', 'S', 'T'
}
```

Missing handlers:
- `'r'` - DECSTBM (Set Top and Bottom Margins)
- `'S'` - SU (Scroll Up)
- `'T'` - SD (Scroll Down)

The existing `ESC M` (Reverse Index) handler also doesn't respect scroll regions.

## Decision

### Add Scroll Region Tracking

Add two fields to `TerminalBuffer`:
```rust
pub struct TerminalBuffer {
    // ... existing fields ...
    /// Top margin of scroll region (0-indexed, inclusive)
    scroll_top: usize,
    /// Bottom margin of scroll region (0-indexed, inclusive)
    scroll_bottom: usize,
}
```

Initialize to full screen in `new()`:
```rust
scroll_top: 0,
scroll_bottom: height - 1,
```

### Implement CSI r (DECSTBM)

```rust
'r' => {
    // DECSTBM - Set Top and Bottom Margins
    let top = params.first().copied().unwrap_or(1) as usize;
    let bottom = params.get(1).copied().unwrap_or(self.height as u16) as usize;

    // Convert to 0-indexed and clamp
    self.scroll_top = top.saturating_sub(1).min(self.height - 1);
    self.scroll_bottom = bottom.saturating_sub(1).min(self.height - 1);

    // Ensure top < bottom
    if self.scroll_top >= self.scroll_bottom {
        // Invalid region, reset to full screen
        self.scroll_top = 0;
        self.scroll_bottom = self.height - 1;
    }

    // Move cursor to home position (per DECSTBM spec)
    self.cursor_row = 0;
    self.cursor_col = 0;
}
```

### Implement CSI S (Scroll Up)

```rust
'S' => {
    // SU - Scroll Up
    let n = params.first().copied().unwrap_or(1).max(1) as usize;
    self.scroll_up(n);
}

fn scroll_up(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;

    for _ in 0..n {
        // Remove top line of region
        self.buffer.remove(top);
        // Insert blank line at bottom of region
        self.buffer.insert(bottom, vec![Cell::default(); self.width]);
    }
}
```

### Implement CSI T (Scroll Down)

```rust
'T' => {
    // SD - Scroll Down
    let n = params.first().copied().unwrap_or(1).max(1) as usize;
    self.scroll_down(n);
}

fn scroll_down(&mut self, n: usize) {
    let top = self.scroll_top;
    let bottom = self.scroll_bottom;

    for _ in 0..n {
        // Remove bottom line of region
        self.buffer.remove(bottom);
        // Insert blank line at top of region
        self.buffer.insert(top, vec![Cell::default(); self.width]);
    }
}
```

### Update ESC M (Reverse Index)

Current implementation doesn't respect scroll region:
```rust
b'M' => {
    // RI - Reverse Index (move cursor up, scroll if at top)
    if *self.cursor_row > 0 {
        *self.cursor_row -= 1;
    } else {
        // Scroll down - add empty row at top, remove bottom
        self.buffer.pop();
        self.buffer.insert(0, vec![Cell::default(); self.width]);
    }
}
```

Updated to respect scroll region:
```rust
b'M' => {
    if *self.cursor_row > self.scroll_top {
        *self.cursor_row -= 1;
    } else if *self.cursor_row == self.scroll_top {
        // At top of scroll region - scroll down within region
        self.scroll_down(1);
    }
    // If cursor is above scroll region, just move up (no scroll)
}
```

### Update line_feed (Newline Handling)

The `line_feed` method should also respect scroll regions. Currently it scrolls the entire screen when at the bottom:

```rust
fn line_feed(&mut self) {
    if *self.cursor_row + 1 < self.height {
        *self.cursor_row += 1;
    } else {
        // Scroll up - remove first row and add empty row at bottom
        self.buffer.remove(0);
        self.buffer.push(vec![Cell::default(); self.width]);
    }
}
```

Updated to respect scroll region:
```rust
fn line_feed(&mut self) {
    if *self.cursor_row < self.scroll_bottom {
        *self.cursor_row += 1;
    } else if *self.cursor_row == self.scroll_bottom {
        // At bottom of scroll region - scroll up within region
        self.scroll_up(1);
    }
    // If cursor is below scroll region, just move down (no scroll)
}
```

### Update Resize Handler

Reset scroll region on resize:
```rust
pub fn resize(&mut self, new_width: usize, new_height: usize) {
    // ... existing resize logic ...

    // Reset scroll region to full screen
    self.scroll_top = 0;
    self.scroll_bottom = new_height - 1;
}
```

## Consequences

### Positive
- Playback matches asciinema/standard terminal emulators
- TUI app recordings render correctly
- No more "corrupted file" false alarms from users

### Negative
- More complex TerminalBuffer implementation
- Need to update existing scroll-related code to respect regions

### Risks
- Edge cases in scroll region behavior may differ from terminals
- Performance impact of more complex scroll operations (likely negligible)

## Testing Strategy

1. **Unit tests** for new handlers:
   - CSI r with various params (default, explicit, invalid)
   - CSI S and T scroll operations
   - ESC M respecting scroll region

2. **Visual comparison test** (already added):
   - Compare output with pyte at multiple checkpoints
   - Verify line positions match

3. **Regression tests**:
   - Existing terminal tests must still pass
   - No change to non-scroll-region recordings
