//! Keyboard input handling for the native player.
//!
//! Handles all keyboard shortcuts including playback controls,
//! navigation, mode toggles, and seeking.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::asciicast::AsciicastFile;
use crate::player::playback::{find_event_index_at_time, seek_to_time};
use crate::player::state::{InputResult, MarkerPosition, PlaybackState};
use crate::terminal::TerminalBuffer;

/// Handle a keyboard event.
///
/// This is the main keyboard input handler that processes all key events
/// and updates state or returns control flow signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_key_event(
    key: KeyEvent,
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    markers: &[MarkerPosition],
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) -> InputResult {
    // If help is showing, any key closes it
    if state.show_help {
        state.show_help = false;
        state.needs_render = true;
        return InputResult::Continue;
    }

    match key.code {
        // === Quit ===
        KeyCode::Char('q') => InputResult::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => InputResult::Quit,
        KeyCode::Esc => {
            if state.exit_mode_or_quit() {
                InputResult::Continue
            } else {
                InputResult::Quit
            }
        }

        // === Mode toggles ===
        KeyCode::Char('?') => {
            state.toggle_help();
            InputResult::Continue
        }
        KeyCode::Char('v') => {
            state.toggle_viewport_mode();
            InputResult::Continue
        }
        KeyCode::Char('f') => {
            state.toggle_free_mode(buffer.cursor_row());
            InputResult::Continue
        }

        // === Playback controls ===
        KeyCode::Char(' ') => {
            state.toggle_pause();
            InputResult::Continue
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            state.speed_up();
            InputResult::Continue
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            state.speed_down();
            InputResult::Continue
        }

        // === Resize terminal ===
        KeyCode::Char('r') => {
            handle_resize_to_recording(state, rec_cols, rec_rows);
            InputResult::Continue
        }

        // === Marker navigation ===
        KeyCode::Char('m') => {
            handle_jump_to_marker(state, buffer, cast, markers, rec_cols, rec_rows);
            InputResult::Continue
        }

        // === Seeking ===
        KeyCode::Char('<') | KeyCode::Char(',') => {
            handle_seek_backward(state, buffer, cast, 5.0, rec_cols, rec_rows);
            InputResult::Continue
        }
        KeyCode::Char('>') | KeyCode::Char('.') => {
            handle_seek_forward(state, buffer, cast, 5.0, total_duration, rec_cols, rec_rows);
            InputResult::Continue
        }
        KeyCode::Home => {
            handle_seek_to_start(state, buffer, cast, rec_cols, rec_rows);
            InputResult::Continue
        }
        KeyCode::End => {
            handle_seek_to_end(state, buffer, cast, total_duration, rec_cols, rec_rows);
            InputResult::Continue
        }

        // === Arrow keys (context-dependent) ===
        KeyCode::Left => {
            handle_left_key(
                state,
                buffer,
                cast,
                key.modifiers,
                total_duration,
                rec_cols,
                rec_rows,
            );
            InputResult::Continue
        }
        KeyCode::Right => {
            handle_right_key(
                state,
                buffer,
                cast,
                key.modifiers,
                total_duration,
                rec_cols,
                rec_rows,
            );
            InputResult::Continue
        }
        KeyCode::Up => {
            handle_up_key(state, rec_rows);
            InputResult::Continue
        }
        KeyCode::Down => {
            handle_down_key(state, rec_rows);
            InputResult::Continue
        }

        _ => InputResult::Continue,
    }
}

/// Handle resize terminal to match recording size.
fn handle_resize_to_recording(state: &mut PlaybackState, rec_cols: u32, rec_rows: u32) {
    // NOTE: This uses xterm escape sequence which only works on
    // xterm-compatible terminals (iTerm2, xterm, etc.)
    let target_rows = rec_rows + PlaybackState::STATUS_LINES as u32;
    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1b[8;{};{}t", target_rows, rec_cols);
    let _ = stdout.flush();

    // Small delay for terminal to resize
    std::thread::sleep(Duration::from_millis(50));

    // Update view dimensions after resize
    if let Ok((new_cols, new_rows)) = crossterm::terminal::size() {
        state.term_cols = new_cols;
        state.term_rows = new_rows;
        state.view_rows = (new_rows.saturating_sub(PlaybackState::STATUS_LINES)) as usize;
        state.view_cols = new_cols as usize;

        // Check if resize succeeded (terminal at least as big as recording)
        let resize_ok = new_cols as u32 >= rec_cols
            && new_rows >= PlaybackState::STATUS_LINES + rec_rows as u16;
        if resize_ok {
            // Reset viewport offset since we now fit
            if state.view_rows >= rec_rows as usize {
                state.view_row_offset = 0;
            }
            if state.view_cols >= rec_cols as usize {
                state.view_col_offset = 0;
            }
        }
    }
    state.needs_render = true;
}

/// Handle jump to next marker.
fn handle_jump_to_marker(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    markers: &[MarkerPosition],
    rec_cols: u32,
    rec_rows: u32,
) {
    if let Some(next) = markers.iter().find(|m| m.time > state.current_time + 0.1) {
        seek_to_time(buffer, cast, next.time, rec_cols, rec_rows);
        state.current_time = next.time;
        state.time_offset = state.current_time;
        (state.event_idx, state.cumulative_time) =
            find_event_index_at_time(cast, state.current_time);
        state.paused = true;
        state.needs_render = true;
    }
}

/// Handle seeking backward by a given amount.
fn handle_seek_backward(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    amount: f64,
    rec_cols: u32,
    rec_rows: u32,
) {
    let new_time = (state.current_time - amount).max(0.0);
    seek_to_time(buffer, cast, new_time, rec_cols, rec_rows);
    state.current_time = new_time;
    state.time_offset = state.current_time;
    state.start_time = Instant::now();
    (state.event_idx, state.cumulative_time) = find_event_index_at_time(cast, state.current_time);
    state.needs_render = true;
}

/// Handle seeking forward by a given amount.
fn handle_seek_forward(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    amount: f64,
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) {
    let new_time = (state.current_time + amount).min(total_duration);
    state.current_time = new_time;
    state.time_offset = state.current_time;
    state.start_time = Instant::now();
    (state.event_idx, state.cumulative_time) = find_event_index_at_time(cast, state.current_time);

    // Rebuild buffer from scratch for forward seek
    *buffer = TerminalBuffer::new(rec_cols as usize, rec_rows as usize);
    let mut cumulative = 0.0f64;
    for event in &cast.events {
        cumulative += event.time;
        if cumulative > state.current_time {
            break;
        }
        if event.is_output() {
            buffer.process(&event.data);
        } else if let Some((cols, rows)) = event.parse_resize() {
            buffer.resize(cols as usize, rows as usize);
        }
    }
    state.needs_render = true;
}

/// Handle seek to start of recording.
fn handle_seek_to_start(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    rec_cols: u32,
    rec_rows: u32,
) {
    seek_to_time(buffer, cast, 0.0, rec_cols, rec_rows);
    state.current_time = 0.0;
    state.time_offset = 0.0;
    state.start_time = Instant::now();
    state.event_idx = 0;
    state.cumulative_time = 0.0;
    state.view_row_offset = 0;
    state.view_col_offset = 0;
    state.needs_render = true;
}

/// Handle seek to end of recording.
fn handle_seek_to_end(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) {
    *buffer = TerminalBuffer::new(rec_cols as usize, rec_rows as usize);

    // Process all events
    for event in &cast.events {
        if event.is_output() {
            buffer.process(&event.data);
        } else if let Some((cols, rows)) = event.parse_resize() {
            buffer.resize(cols as usize, rows as usize);
        }
    }

    state.current_time = total_duration;
    state.time_offset = state.current_time;
    state.event_idx = cast.events.len();
    state.cumulative_time = total_duration;
    state.paused = true;
    state.needs_render = true;
}

/// Handle left arrow key (seek or viewport scroll).
fn handle_left_key(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    modifiers: KeyModifiers,
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) {
    if state.viewport_mode {
        state.view_col_offset = state.view_col_offset.saturating_sub(1);
        state.needs_render = true;
    } else {
        let step = if modifiers.contains(KeyModifiers::SHIFT) {
            total_duration * 0.05 // 5% jump
        } else {
            5.0 // 5 seconds
        };
        handle_seek_backward(state, buffer, cast, step, rec_cols, rec_rows);
    }
}

/// Handle right arrow key (seek or viewport scroll).
fn handle_right_key(
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    modifiers: KeyModifiers,
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) {
    if state.viewport_mode {
        let max_offset = (rec_cols as usize).saturating_sub(state.view_cols);
        state.view_col_offset = (state.view_col_offset + 1).min(max_offset);
        state.needs_render = true;
    } else {
        let step = if modifiers.contains(KeyModifiers::SHIFT) {
            total_duration * 0.05 // 5% jump
        } else {
            5.0 // 5 seconds
        };
        handle_seek_forward(
            state,
            buffer,
            cast,
            step,
            total_duration,
            rec_cols,
            rec_rows,
        );
    }
}

/// Handle up arrow key (free mode or viewport scroll).
fn handle_up_key(state: &mut PlaybackState, _rec_rows: u32) {
    if state.free_mode {
        // Move highlight up one line
        let old_offset = state.view_row_offset;
        state.prev_free_line = state.free_line;
        state.free_line = state.free_line.saturating_sub(1);

        // Auto-scroll viewport to keep highlighted line visible
        if state.free_line < state.view_row_offset {
            state.view_row_offset = state.free_line;
        }

        // If viewport didn't scroll, only update highlight lines
        if state.view_row_offset == old_offset && state.prev_free_line != state.free_line {
            state.free_line_only = true;
        }
        state.needs_render = true;
    } else if state.viewport_mode {
        state.view_row_offset = state.view_row_offset.saturating_sub(1);
        state.needs_render = true;
    }
    // In normal mode, up does nothing
}

/// Handle down arrow key (free mode or viewport scroll).
fn handle_down_key(state: &mut PlaybackState, rec_rows: u32) {
    if state.free_mode {
        // Move highlight down one line
        let old_offset = state.view_row_offset;
        state.prev_free_line = state.free_line;
        let max_line = (rec_rows as usize).saturating_sub(1);
        state.free_line = (state.free_line + 1).min(max_line);

        // Auto-scroll viewport to keep highlighted line visible
        if state.free_line >= state.view_row_offset + state.view_rows {
            state.view_row_offset = state.free_line - state.view_rows + 1;
        }

        // If viewport didn't scroll, only update highlight lines
        if state.view_row_offset == old_offset && state.prev_free_line != state.free_line {
            state.free_line_only = true;
        }
        state.needs_render = true;
    } else if state.viewport_mode {
        let max_offset = (rec_rows as usize).saturating_sub(state.view_rows);
        state.view_row_offset = (state.view_row_offset + 1).min(max_offset);
        state.needs_render = true;
    }
    // In normal mode, down does nothing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_up_key_free_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.free_mode = true;
        state.free_line = 5;

        handle_up_key(&mut state, 24);

        assert_eq!(state.free_line, 4);
        assert_eq!(state.prev_free_line, 5);
    }

    #[test]
    fn test_handle_down_key_free_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.free_mode = true;
        state.free_line = 5;

        handle_down_key(&mut state, 24);

        assert_eq!(state.free_line, 6);
        assert_eq!(state.prev_free_line, 5);
    }

    #[test]
    fn test_handle_up_key_viewport_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.viewport_mode = true;
        state.view_row_offset = 5;

        handle_up_key(&mut state, 48);

        assert_eq!(state.view_row_offset, 4);
    }

    #[test]
    fn test_handle_down_key_viewport_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.viewport_mode = true;
        state.view_row_offset = 5;

        handle_down_key(&mut state, 48);

        assert_eq!(state.view_row_offset, 6);
    }
}
