//! Input handling for the native player.
//!
//! This module handles keyboard and mouse input events, dispatching
//! them to the appropriate handlers and returning control flow signals.

mod keyboard;
mod mouse;

pub use keyboard::handle_key_event;
pub use mouse::handle_mouse_event;

use crate::player::state::InputResult;
use crossterm::event::Event;

use crate::asciicast::AsciicastFile;
use crate::player::state::{MarkerPosition, PlaybackState};
use crate::terminal::TerminalBuffer;

/// Handle any input event, dispatching to the appropriate handler.
///
/// # Arguments
/// * `event` - The crossterm event to handle
/// * `state` - Mutable reference to playback state
/// * `buffer` - Mutable reference to terminal buffer (for seeking)
/// * `cast` - Reference to the cast file
/// * `markers` - Reference to collected markers
/// * `total_duration` - Total duration of the recording
/// * `rec_cols` - Recording width
/// * `rec_rows` - Recording height
///
/// # Returns
/// `InputResult` indicating whether to continue, quit, or quit with file
#[allow(clippy::too_many_arguments)]
pub fn handle_event(
    event: Event,
    state: &mut PlaybackState,
    buffer: &mut TerminalBuffer,
    cast: &AsciicastFile,
    markers: &[MarkerPosition],
    total_duration: f64,
    rec_cols: u32,
    rec_rows: u32,
) -> InputResult {
    match event {
        Event::Key(key) => handle_key_event(
            key,
            state,
            buffer,
            cast,
            markers,
            total_duration,
            rec_cols,
            rec_rows,
        ),
        Event::Mouse(mouse) => handle_mouse_event(
            mouse,
            state,
            buffer,
            cast,
            total_duration,
            rec_cols,
            rec_rows,
        ),
        Event::Resize(new_cols, new_rows) => {
            state.handle_resize(new_cols, new_rows, rec_cols, rec_rows);
            InputResult::Continue
        }
        _ => InputResult::Continue, // Ignore focus events, etc.
    }
}
