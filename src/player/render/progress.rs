//! Progress bar rendering for the native player.
//!
//! Displays playback progress with marker indicators.

use std::io::{self, Write};

use anyhow::Result;

use crate::player::state::MarkerPosition;

/// Format a duration in seconds to MM:SS format.
///
/// # Arguments
/// * `seconds` - Duration in seconds
///
/// # Returns
/// A string in MM:SS format
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Build the progress bar character array.
///
/// Creates a visual representation of the progress bar including
/// the playhead position and marker indicators.
///
/// # Arguments
/// * `bar_width` - Width of the bar in characters
/// * `current_time` - Current playback time
/// * `total_duration` - Total duration of the recording
/// * `markers` - Slice of marker positions
///
/// # Returns
/// A tuple of (bar_chars, filled_count) where bar_chars contains the visual
/// representation and filled_count is the number of filled positions.
pub fn build_progress_bar_chars(
    bar_width: usize,
    current_time: f64,
    total_duration: f64,
    markers: &[MarkerPosition],
) -> (Vec<char>, usize) {
    let progress = if total_duration > 0.0 {
        (current_time / total_duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let filled = (bar_width as f64 * progress) as usize;

    let mut bar: Vec<char> = vec!['─'; bar_width];

    if filled < bar_width {
        bar[filled] = '⏺';
    }

    for marker in markers {
        let marker_pos = if total_duration > 0.0 {
            ((marker.time / total_duration) * bar_width as f64) as usize
        } else {
            0
        };
        if marker_pos < bar_width && bar[marker_pos] != '⏺' {
            bar[marker_pos] = '◆';
        }
    }

    (bar, filled)
}

/// Render the progress bar with markers.
///
/// # Arguments
/// * `stdout` - The stdout handle to write to
/// * `width` - Terminal width
/// * `row` - Row to render at (0-indexed)
/// * `current_time` - Current playback time
/// * `total_duration` - Total duration of the recording
/// * `markers` - Slice of marker positions
pub fn render_progress_bar(
    stdout: &mut io::Stdout,
    width: u16,
    row: u16,
    current_time: f64,
    total_duration: f64,
    markers: &[MarkerPosition],
) -> Result<()> {
    let bar_width = (width as usize).saturating_sub(14); // Account for padding and time display
    let (bar, filled) = build_progress_bar_chars(bar_width, current_time, total_duration, markers);

    let current_str = format_duration(current_time);
    let total_str = format_duration(total_duration);
    let time_display = format!(" {}/{}", current_str, total_str);

    // Build output string
    let mut output = String::with_capacity(width as usize * 4);
    output.push_str(&format!("\x1b[{};1H", row + 1)); // Move cursor
    output.push_str("\x1b[48;5;236m "); // Dark gray background + padding

    // ANSI color codes
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const WHITE: &str = "\x1b[97m";
    const DARK_GREY: &str = "\x1b[90m";
    const GREY: &str = "\x1b[37m";

    output.push_str(GREEN);
    for (i, &c) in bar.iter().enumerate() {
        if i < filled {
            if c == '◆' {
                output.push_str(YELLOW);
                output.push(c);
                output.push_str(GREEN);
            } else {
                output.push('━');
            }
        } else if i == filled {
            output.push_str(WHITE);
            output.push(c);
        } else if c == '◆' {
            output.push_str(YELLOW);
            output.push(c);
        } else {
            output.push_str(DARK_GREY);
            output.push(c);
        }
    }

    output.push_str(GREY);
    output.push_str(&time_display);

    // Fill remaining width
    let used_width = 1 + bar_width + time_display.len();
    let remaining = (width as usize).saturating_sub(used_width);
    for _ in 0..remaining {
        output.push(' ');
    }

    output.push_str("\x1b[0m"); // Reset
    write!(stdout, "{}", output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_formats_correctly() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3661.0), "61:01");
    }

    #[test]
    fn format_duration_edge_cases() {
        // Fractional seconds are truncated
        assert_eq!(format_duration(0.9), "00:00");
        assert_eq!(format_duration(1.5), "00:01");
        assert_eq!(format_duration(59.9), "00:59");
        // Very large durations (hours)
        assert_eq!(format_duration(7200.0), "120:00"); // 2 hours
    }

    #[test]
    fn format_duration_negative_treated_as_zero() {
        // Negative durations should still format (as 0 due to u64 cast)
        assert_eq!(format_duration(-5.0), "00:00");
    }

    #[test]
    fn empty_bar_at_zero() {
        let (bar, filled) = build_progress_bar_chars(10, 0.0, 10.0, &[]);
        assert_eq!(filled, 0);
        assert_eq!(bar[0], '⏺'); // Playhead at start
        assert_eq!(bar[1], '─');
    }

    #[test]
    fn full_bar_at_end() {
        let (bar, filled) = build_progress_bar_chars(10, 10.0, 10.0, &[]);
        assert_eq!(filled, 10);
        // All positions should be regular bar chars (no playhead since filled == bar_width)
        assert!(bar.iter().all(|&c| c == '─'));
    }

    #[test]
    fn half_progress() {
        let (bar, filled) = build_progress_bar_chars(10, 5.0, 10.0, &[]);
        assert_eq!(filled, 5);
        assert_eq!(bar[5], '⏺'); // Playhead at middle
    }

    #[test]
    fn marker_at_position() {
        let markers = vec![MarkerPosition {
            time: 5.0,
            label: "test".to_string(),
        }];
        let (bar, _) = build_progress_bar_chars(10, 0.0, 10.0, &markers);
        assert_eq!(bar[5], '◆'); // Marker at position 5
    }

    #[test]
    fn marker_not_overwritten_by_playhead() {
        // Marker at same position as playhead - playhead wins
        let markers = vec![MarkerPosition {
            time: 5.0,
            label: "test".to_string(),
        }];
        let (bar, _) = build_progress_bar_chars(10, 5.0, 10.0, &markers);
        assert_eq!(bar[5], '⏺'); // Playhead takes precedence
    }

    #[test]
    fn multiple_markers() {
        let markers = vec![
            MarkerPosition {
                time: 2.0,
                label: "m1".to_string(),
            },
            MarkerPosition {
                time: 8.0,
                label: "m2".to_string(),
            },
        ];
        let (bar, _) = build_progress_bar_chars(10, 0.0, 10.0, &markers);
        assert_eq!(bar[2], '◆');
        assert_eq!(bar[8], '◆');
    }

    #[test]
    fn zero_duration_returns_full() {
        let (_, filled) = build_progress_bar_chars(10, 5.0, 0.0, &[]);
        assert_eq!(filled, 10); // progress = 1.0 when duration is 0
    }

    #[test]
    fn progress_clamped_to_one() {
        // Current time exceeds total duration
        let (_, filled) = build_progress_bar_chars(10, 15.0, 10.0, &[]);
        assert_eq!(filled, 10); // Clamped to 100%
    }

    #[test]
    fn marker_at_zero_duration() {
        let markers = vec![MarkerPosition {
            time: 5.0,
            label: "m".to_string(),
        }];
        let (bar, _) = build_progress_bar_chars(10, 0.0, 0.0, &markers);
        // When duration is 0, marker_pos = 0
        assert_eq!(bar[0], '◆');
    }
}
