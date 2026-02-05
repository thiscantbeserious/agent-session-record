//! Whitespace normalization and empty event filtering transforms.
//!
//! These transforms reduce noise from excessive whitespace and empty events.

use crate::asciicast::{Event, Transform};

/// Normalizes excessive whitespace in event content.
///
/// - Collapses multiple consecutive spaces to a single space
/// - Limits consecutive newlines to a configurable maximum
pub struct NormalizeWhitespace {
    max_consecutive_newlines: usize,
}

impl NormalizeWhitespace {
    /// Create a new whitespace normalizer.
    pub fn new(max_consecutive_newlines: usize) -> Self {
        Self {
            max_consecutive_newlines,
        }
    }
}

impl Default for NormalizeWhitespace {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Transform for NormalizeWhitespace {
    fn transform(&mut self, events: &mut Vec<Event>) {
        for event in events.iter_mut() {
            if event.is_output() {
                let mut result = String::with_capacity(event.data.len());
                let mut prev_space = false;
                let mut newline_count = 0;

                for c in event.data.chars() {
                    if c == '\n' {
                        newline_count += 1;
                        if newline_count <= self.max_consecutive_newlines {
                            result.push(c);
                        }
                        prev_space = false;
                    } else if c == ' ' || c == '\t' {
                        newline_count = 0;
                        if !prev_space {
                            result.push(' ');
                            prev_space = true;
                        }
                    } else {
                        newline_count = 0;
                        prev_space = false;
                        result.push(c);
                    }
                }
                event.data = result;
            }
        }
    }
}

/// Filters out events with no content.
///
/// Removes output events that are empty or contain only whitespace.
/// **Always preserves**: markers, input events, resize events.
pub struct FilterEmptyEvents;

impl Transform for FilterEmptyEvents {
    fn transform(&mut self, events: &mut Vec<Event>) {
        events.retain(|event| {
            // Always keep non-output events (markers, input, resize)
            if !event.is_output() {
                return true;
            }
            // Keep output events only if they have non-whitespace content
            !event.data.trim().is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NormalizeWhitespace tests

    #[test]
    fn collapses_multiple_spaces() {
        let mut normalizer = NormalizeWhitespace::new(2);
        let mut events = vec![Event::output(0.1, "hello    world")];

        normalizer.transform(&mut events);

        assert_eq!(events[0].data, "hello world");
    }

    #[test]
    fn limits_consecutive_newlines() {
        let mut normalizer = NormalizeWhitespace::new(2);
        let mut events = vec![Event::output(0.1, "line1\n\n\n\n\nline2")];

        normalizer.transform(&mut events);

        assert_eq!(events[0].data, "line1\n\nline2");
    }

    #[test]
    fn converts_tabs_to_space() {
        let mut normalizer = NormalizeWhitespace::new(2);
        let mut events = vec![Event::output(0.1, "hello\t\tworld")];

        normalizer.transform(&mut events);

        assert_eq!(events[0].data, "hello world");
    }

    // FilterEmptyEvents tests

    #[test]
    fn removes_empty_events() {
        let mut events = vec![
            Event::output(0.1, "hello"),
            Event::output(0.1, ""),
            Event::output(0.1, "world"),
        ];

        FilterEmptyEvents.transform(&mut events);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn removes_whitespace_only_events() {
        let mut events = vec![
            Event::output(0.1, "hello"),
            Event::output(0.1, "   \n\t  "),
            Event::output(0.1, "world"),
        ];

        FilterEmptyEvents.transform(&mut events);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn preserves_markers() {
        let mut events = vec![
            Event::output(0.1, ""),
            Event::marker(0.1, "marker"),
            Event::output(0.1, ""),
        ];

        FilterEmptyEvents.transform(&mut events);

        assert_eq!(events.len(), 1);
        assert!(events[0].is_marker());
    }
}
