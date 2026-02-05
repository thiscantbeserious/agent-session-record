//! Progress line deduplication transform.
//!
//! Terminal progress bars often use carriage return (`\r`) to rewrite the same
//! line thousands of times. This transform keeps only the final state of each
//! line, dramatically reducing content size while preserving meaning.

use crate::asciicast::{Event, Transform};

/// Deduplicates progress lines that use `\r` to overwrite themselves.
///
/// **Algorithm**:
/// 1. Track "current line buffer" with timestamp of FIRST char
/// 2. When `\r` is encountered, clear buffer but keep timestamp
/// 3. When `\n` is encountered, emit the line with timestamp of line START
/// 4. Non-output events (markers, input) pass through unchanged
pub struct DeduplicateProgressLines {
    current_line: String,
    line_start_time: f64,
    is_progress_line: bool,
    deduped_count: usize,
}

impl DeduplicateProgressLines {
    /// Create a new progress line deduplicator.
    pub fn new() -> Self {
        Self {
            current_line: String::new(),
            line_start_time: 0.0,
            is_progress_line: false,
            deduped_count: 0,
        }
    }

    /// Get the count of deduplicated progress lines.
    pub fn deduped_count(&self) -> usize {
        self.deduped_count
    }
}

impl Default for DeduplicateProgressLines {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for DeduplicateProgressLines {
    fn transform(&mut self, events: &mut Vec<Event>) {
        let mut output_events = Vec::with_capacity(events.len());

        // Track cumulative time for absolute timestamps
        let mut cumulative_time = 0.0;

        for event in events.drain(..) {
            cumulative_time += event.time;

            // Preserve non-output events (markers, input, resize)
            if !event.is_output() {
                // Emit any pending line content before the marker
                if !self.current_line.is_empty() {
                    output_events.push(Event::output(
                        self.line_start_time,
                        std::mem::take(&mut self.current_line),
                    ));
                }
                output_events.push(event);
                continue;
            }

            for ch in event.data.chars() {
                match ch {
                    '\r' => {
                        // Carriage return: line will be overwritten
                        self.is_progress_line = true;
                        self.current_line.clear();
                        // Update start time to current event time
                        self.line_start_time = cumulative_time;
                    }
                    '\n' => {
                        // Newline: emit current line if not empty
                        if !self.current_line.is_empty() {
                            output_events.push(Event::output(
                                self.line_start_time,
                                format!("{}\n", self.current_line),
                            ));
                        } else {
                            // Emit standalone newline
                            output_events.push(Event::output(cumulative_time, "\n".to_string()));
                        }
                        if self.is_progress_line {
                            self.deduped_count += 1;
                        }
                        self.current_line.clear();
                        self.is_progress_line = false;
                    }
                    _ => {
                        // First char of new line sets the timestamp
                        if self.current_line.is_empty() {
                            self.line_start_time = cumulative_time;
                        }
                        self.current_line.push(ch);
                    }
                }
            }
        }

        // Don't forget trailing content without \n
        if !self.current_line.is_empty() {
            output_events.push(Event::output(
                self.line_start_time,
                std::mem::take(&mut self.current_line),
            ));
        }

        *events = output_events;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_cr_lines() {
        let mut deduper = DeduplicateProgressLines::new();
        let mut events = vec![
            Event::output(0.1, "\r⠋ Building..."),
            Event::output(0.1, "\r⠙ Building..."),
            Event::output(0.1, "\r⠹ Building..."),
            Event::output(0.1, "\r✓ Build complete\n"),
        ];

        deduper.transform(&mut events);

        // Should have one event with final content
        assert_eq!(events.len(), 1);
        assert!(events[0].data.contains("Build complete"));
    }

    #[test]
    fn preserves_markers() {
        let mut deduper = DeduplicateProgressLines::new();
        let mut events = vec![
            Event::output(0.1, "line1\n"),
            Event::marker(0.1, "marker"),
            Event::output(0.1, "line2\n"),
        ];

        deduper.transform(&mut events);

        // Marker should be preserved in order
        let markers: Vec<_> = events.iter().filter(|e| e.is_marker()).collect();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].data, "marker");
    }

    #[test]
    fn preserves_non_progress_lines() {
        let mut deduper = DeduplicateProgressLines::new();
        let mut events = vec![
            Event::output(0.1, "first line\n"),
            Event::output(0.1, "second line\n"),
            Event::output(0.1, "third line\n"),
        ];

        deduper.transform(&mut events);

        // All three lines should be preserved
        let content: String = events.iter().map(|e| e.data.as_str()).collect();
        assert!(content.contains("first line"));
        assert!(content.contains("second line"));
        assert!(content.contains("third line"));
    }
}
