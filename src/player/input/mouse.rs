//! Mouse input handling for the native player.
//!
//! Handles mouse events, primarily for click-to-seek on the progress bar.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;

use crate::asciicast::AsciicastFile;
use crate::player::playback::{find_event_index_at_time, seek_to_time};
use crate::player::state::{InputResult, PlaybackState};
use crate::terminal::TerminalBuffer;

/// Handle a mouse event.
///
/// Currently handles:
/// - Left click on progress bar to seek to that position
#[allow(clippy::too_many_arguments)]
pub fn handle_mouse_event(
    mouse: MouseEvent,
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) -> InputResult {
    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        let progress_row = state.term_rows - 2;

        if mouse.row == progress_row {
            // Calculate time from x position on progress bar
            // Bar starts at column 1, width is term_cols - 14
            let bar_start = 1u16;
            let bar_width = (state.term_cols as usize).saturating_sub(14);

            if mouse.column >= bar_start && mouse.column < bar_start + bar_width as u16 {
                let click_pos = (mouse.column - bar_start) as f64;
                let ratio = click_pos / bar_width as f64;
                let new_time = (ratio * total_duration).clamp(0.0, total_duration);

                // Exit free mode if active
                state.free_mode = false;

                // Seek to clicked position
                seek_to_time(buffer, cast, new_time, rec_cols, rec_rows);
                state.current_time = new_time;
                state.time_offset = state.current_time;
                state.start_time = Instant::now();
                (state.event_idx, state.cumulative_time) =
                    find_event_index_at_time(cast, state.current_time);

                // Resume playback after seeking
                state.paused = false;
                state.needs_render = true;
            }
        }
    }

    InputResult::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mouse handling is primarily tested through integration tests
    // since it depends on terminal state and crossterm events

    #[test]
    fn test_progress_bar_position_calculation() {
        // Test the progress bar position calculation logic
        let term_cols: u16 = 80;
        let bar_start: u16 = 1;
        let bar_width = (term_cols as usize).saturating_sub(14); // 66

        // Click at start of bar
        let click_pos = (bar_start - bar_start) as f64;
        let ratio = click_pos / bar_width as f64;
        assert_eq!(ratio, 0.0);

        // Click at end of bar
        let click_at_end = bar_start + bar_width as u16 - 1;
        let click_pos = (click_at_end - bar_start) as f64;
        let ratio = click_pos / bar_width as f64;
        assert!((ratio - 0.984).abs() < 0.01); // ~98.4% through the bar
    }
}
