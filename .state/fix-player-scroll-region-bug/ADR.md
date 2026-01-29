# ADR: Fix Player Scroll Region Bug

## Status
Proposed

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
    if *self.cursor_row > 0 {
        *self.cursor_row -= 1;
    } else {
        // Scroll down at top - currently scrolls whole screen
        self.buffer.remove(self.height - 1);
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
