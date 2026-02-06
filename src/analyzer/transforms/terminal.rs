//! Terminal emulation transform.
//!
//! Uses a virtual terminal buffer to process events, handling ANSI escape
//! sequences and carriage return overwrites correctly. This produces a
//! "rendered" version of the terminal state, which is much cleaner for
//! TUI sessions and preserves spatial layout (indentation).

use crate::asciicast::{Event, EventType, Transform};
use crate::terminal::TerminalBuffer;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

/// Maximum number of line hashes to retain. Limits memory for long sessions
/// while still catching redraws within a ~50K-line window. Each entry is 8
/// bytes, so 50 000 entries ≈ 400 KB.
const MAX_STORY_HASHES: usize = 50_000;

/// A transform that renders events through a virtual terminal and extracts
/// a clean, deduplicated chronological "story" of the session.
pub struct TerminalTransform {
    buffer: TerminalBuffer,
    /// Number of stable lines already emitted from the current buffer state
    stable_lines_count: usize,
    /// Last cursor position to detect and skip typing increments
    last_cursor_pos: (usize, usize),
    /// Hashes of lines already included in the stable story to prevent duplicates from redraws
    story_hashes: HashSet<u64>,
    /// Insertion order for FIFO eviction of story_hashes
    story_hash_order: VecDeque<u64>,
}

impl TerminalTransform {
    /// Create a new terminal transform with given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: TerminalBuffer::new(width, height),
            stable_lines_count: 0,
            last_cursor_pos: (0, 0),
            story_hashes: HashSet::with_capacity(MAX_STORY_HASHES),
            story_hash_order: VecDeque::with_capacity(MAX_STORY_HASHES),
        }
    }

    /// Check if a line is "razzle dazzle" thinking noise or status bar.
    fn is_noise(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Target specific TUI status patterns
        trimmed.contains("Shimmying…")
            || trimmed.contains("Orbiting…")
            || trimmed.contains("Improvising…")
            || trimmed.contains("Whatchamacalliting…")
            || trimmed.contains("Churning…")
            || trimmed.contains("Clauding…")
            || trimmed.contains("Razzle-dazzling…")
            || trimmed.contains("Wibbling…")
            || trimmed.contains("Bloviating…")
            || trimmed.contains("Herding…")
            || trimmed.contains("Channeling…")
            || trimmed.contains("Unfurling…")
            || trimmed.contains("accept edits on (shift+Tab to cycle)")
            || trimmed.contains("Context left until auto-compact")
            || trimmed.contains("thinking")
            || trimmed.contains("Tip:")
            || trimmed.contains("Update available!")
            || (trimmed.contains("Done") && trimmed.contains("tool uses"))
    }

    fn hash_line(line: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // We trim trailing whitespace for hashing to treat redraws with different
        // padding as identical, but we preserve leading whitespace for indentation.
        line.trim_end().hash(&mut hasher);
        hasher.finish()
    }

    /// Insert a hash with bounded FIFO eviction.
    fn insert_hash(&mut self, h: u64) -> bool {
        if !self.story_hashes.insert(h) {
            return false; // already seen
        }
        self.story_hash_order.push_back(h);
        // Evict oldest when over capacity
        while self.story_hashes.len() > MAX_STORY_HASHES {
            if let Some(old) = self.story_hash_order.pop_front() {
                self.story_hashes.remove(&old);
            }
        }
        true
    }

    /// Helper to filter and emit lines while updating story_hashes.
    fn filter_new_lines(&mut self, lines: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for line in lines {
            if Self::is_noise(&line) {
                continue;
            }
            let h = Self::hash_line(&line);
            if self.insert_hash(h) {
                result.push(line);
            }
        }
        result
    }
}

impl Transform for TerminalTransform {
    fn transform(&mut self, events: &mut Vec<Event>) {
        let mut output_events = Vec::with_capacity(events.len());
        let mut accumulated_time = 0.0;

        for event in events.drain(..) {
            match event.event_type {
                EventType::Output => {
                    let mut scrolled_lines = Vec::new();
                    {
                        let mut scroll_cb = |cells: Vec<crate::terminal::Cell>| {
                            let line: String = cells.iter().map(|c| c.char).collect();
                            scrolled_lines.push(line);
                        };
                        self.buffer.process(&event.data, Some(&mut scroll_cb));
                    }
                    accumulated_time += event.time;

                    // 1. Emit lines that were scrolled off the screen immediately
                    let had_scroll = !scrolled_lines.is_empty();
                    if had_scroll {
                        let new_lines = self.filter_new_lines(scrolled_lines);
                        if !new_lines.is_empty() {
                            output_events.push(Event::output(
                                accumulated_time,
                                new_lines.join("\n"),
                            ));
                            accumulated_time = 0.0;
                        }
                    }

                    let current_cursor =
                        (self.buffer.cursor_row(), self.buffer.cursor_col());

                    // Optimization: only snapshot the buffer when something
                    // interesting happened (cursor moved, scroll, newline, or
                    // long pause). Skipping to_string() for typing-within-line
                    // events eliminates the dominant cost on large files.
                    let cursor_moved = current_cursor != self.last_cursor_pos;
                    let has_newline = event.data.contains('\n');
                    let long_pause = event.time > 2.0;

                    if cursor_moved || had_scroll || has_newline || long_pause {
                        let current_display = self.buffer.to_string();
                        let current_lines: Vec<String> =
                            current_display.lines().map(|s| s.to_string()).collect();

                        // Logic: lines ABOVE the cursor are considered stable and finished.
                        let mut lines_to_emit = Vec::new();

                        // 2. Identify lines that the cursor has moved past
                        while self.stable_lines_count < current_cursor.0
                            && self.stable_lines_count < current_lines.len()
                        {
                            lines_to_emit
                                .push(current_lines[self.stable_lines_count].clone());
                            self.stable_lines_count += 1;
                        }

                        // 3. Emit the current line IF it was finalized
                        let is_stable =
                            has_newline || current_cursor.0 < self.last_cursor_pos.0 || long_pause;

                        if is_stable
                            && current_cursor.0 < current_lines.len()
                            && self.stable_lines_count <= current_cursor.0
                        {
                            lines_to_emit
                                .push(current_lines[current_cursor.0].clone());
                            if has_newline {
                                self.stable_lines_count = current_cursor.0 + 1;
                            }
                        }

                        if !lines_to_emit.is_empty() {
                            let new_lines = self.filter_new_lines(lines_to_emit);
                            if !new_lines.is_empty() {
                                output_events.push(Event::output(
                                    accumulated_time,
                                    new_lines.join("\n"),
                                ));
                                accumulated_time = 0.0;
                            }
                        }
                    }

                    self.last_cursor_pos = current_cursor;
                }
                EventType::Resize => {
                    if let Some((w, h)) = event.parse_resize() {
                        self.buffer.resize(w as usize, h as usize);
                        let mut e = event;
                        e.time += accumulated_time;
                        accumulated_time = 0.0;
                        output_events.push(e);
                    }
                }
                _ => {
                    let mut e = event;
                    e.time += accumulated_time;
                    accumulated_time = 0.0;
                    output_events.push(e);
                }
            }
        }

        // Final flush
        let current_display = self.buffer.to_string();
        let current_lines: Vec<String> = current_display
            .lines()
            .map(|s| s.trim_end().to_string())
            .collect();
        let mut final_lines = Vec::new();
        while self.stable_lines_count < current_lines.len() {
            final_lines.push(current_lines[self.stable_lines_count].clone());
            self.stable_lines_count += 1;
        }
        if let Some(text) = {
            let filtered = self.filter_new_lines(final_lines);
            if filtered.is_empty() {
                None
            } else {
                Some(filtered.join("\n"))
            }
        } {
            output_events.push(Event::output(accumulated_time, text));
        }

        *events = output_events;
    }
}
