//! Player state management
//!
//! Contains the central `PlaybackState` struct that holds all playback state,
//! as well as shared types used across player modules.

use std::time::Instant;

/// Result of processing an input event.
///
/// This enum is returned by input handlers to signal control flow
/// decisions to the main loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    /// Continue normal playback/rendering
    Continue,
    /// Exit the player normally
    Quit,
    /// Exit and return the file path (for `agr ls` integration)
    QuitWithFile,
}

/// Marker information for the progress bar.
///
/// Tracks the cumulative time and label for each marker in the recording.
#[derive(Debug, Clone)]
pub struct MarkerPosition {
    /// Cumulative time when the marker occurs
    pub time: f64,
    /// Marker label (from the cast file)
    pub label: String,
}

/// Central playback state for the native player.
///
/// This struct contains all state needed for playback, rendering,
/// and input handling. It is passed to various modules as needed.
#[derive(Debug)]
pub struct PlaybackState {
    // === Playback timing ===
    /// Whether playback is paused
    pub paused: bool,
    /// Playback speed multiplier (1.0 = normal)
    pub speed: f64,
    /// Current event index in the cast file
    pub event_idx: usize,
    /// Current playback time in seconds
    pub current_time: f64,
    /// Cumulative time at current event index
    pub cumulative_time: f64,
    /// Wall clock time when playback started/resumed
    pub start_time: Instant,
    /// Time offset for seeking (added to elapsed wall time)
    pub time_offset: f64,

    // === UI modes ===
    /// Whether help overlay is visible
    pub show_help: bool,
    /// Whether viewport mode is active (arrow keys scroll instead of seek)
    pub viewport_mode: bool,
    /// Whether free mode is active (line-by-line navigation)
    pub free_mode: bool,

    // === Free mode state ===
    /// Current highlighted line in free mode (buffer row)
    pub free_line: usize,
    /// Previous highlighted line (for partial updates)
    pub prev_free_line: usize,
    /// True if only free_line changed (enables partial update optimization)
    pub free_line_only: bool,

    // === Viewport state ===
    /// Current terminal width
    pub term_cols: u16,
    /// Current terminal height
    pub term_rows: u16,
    /// Number of visible content rows (term_rows - status_lines)
    pub view_rows: usize,
    /// Number of visible content columns
    pub view_cols: usize,
    /// Vertical scroll offset into buffer
    pub view_row_offset: usize,
    /// Horizontal scroll offset into buffer
    pub view_col_offset: usize,

    // === Rendering flags ===
    /// True when screen needs to be redrawn
    pub needs_render: bool,
}

impl PlaybackState {
    /// Number of status/chrome lines (separator + progress + status bar)
    pub const STATUS_LINES: u16 = 3;

    /// Create a new PlaybackState with default values.
    ///
    /// # Arguments
    /// * `term_cols` - Terminal width in columns
    /// * `term_rows` - Terminal height in rows
    pub fn new(term_cols: u16, term_rows: u16) -> Self {
        let view_rows = (term_rows.saturating_sub(Self::STATUS_LINES)) as usize;
        let view_cols = term_cols as usize;

        Self {
            // Playback timing
            paused: false,
            speed: 1.0,
            event_idx: 0,
            current_time: 0.0,
            cumulative_time: 0.0,
            start_time: Instant::now(),
            time_offset: 0.0,

            // UI modes
            show_help: false,
            viewport_mode: false,
            free_mode: false,

            // Free mode state
            free_line: 0,
            prev_free_line: 0,
            free_line_only: false,

            // Viewport state
            term_cols,
            term_rows,
            view_rows,
            view_cols,
            view_row_offset: 0,
            view_col_offset: 0,

            // Rendering flags
            needs_render: true,
        }
    }

    /// Handle terminal resize event.
    ///
    /// Updates viewport dimensions and clamps scroll offsets to valid range.
    ///
    /// # Arguments
    /// * `new_cols` - New terminal width
    /// * `new_rows` - New terminal height
    /// * `rec_cols` - Recording width (for clamping)
    /// * `rec_rows` - Recording height (for clamping)
    pub fn handle_resize(&mut self, new_cols: u16, new_rows: u16, rec_cols: u32, rec_rows: u32) {
        self.term_cols = new_cols;
        self.term_rows = new_rows;
        self.view_rows = (new_rows.saturating_sub(Self::STATUS_LINES)) as usize;
        self.view_cols = new_cols as usize;

        // Clamp viewport offset to valid range
        let max_row_offset = (rec_rows as usize).saturating_sub(self.view_rows);
        let max_col_offset = (rec_cols as usize).saturating_sub(self.view_cols);
        self.view_row_offset = self.view_row_offset.min(max_row_offset);
        self.view_col_offset = self.view_col_offset.min(max_col_offset);

        self.needs_render = true;
    }

    /// Toggle pause state and reset timing if resuming.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if !self.paused {
            // Exit free mode when resuming playback
            self.free_mode = false;
            // Reset timing when resuming
            self.start_time = Instant::now();
            self.time_offset = self.current_time;
        }
        self.needs_render = true;
    }

    /// Increase playback speed (max 16x).
    pub fn speed_up(&mut self) {
        self.speed = (self.speed * 1.5).min(16.0);
        self.needs_render = true;
    }

    /// Decrease playback speed (min 0.1x).
    pub fn speed_down(&mut self) {
        self.speed = (self.speed / 1.5).max(0.1);
        self.needs_render = true;
    }

    /// Toggle help overlay visibility.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.needs_render = true;
    }

    /// Toggle viewport mode.
    pub fn toggle_viewport_mode(&mut self) {
        self.viewport_mode = !self.viewport_mode;
        if self.viewport_mode {
            self.free_mode = false; // Exit free mode when entering viewport mode
        }
        self.needs_render = true;
    }

    /// Toggle free mode (pauses playback automatically).
    ///
    /// # Arguments
    /// * `cursor_row` - Current cursor row to start highlight at
    pub fn toggle_free_mode(&mut self, cursor_row: usize) {
        self.free_mode = !self.free_mode;
        if self.free_mode {
            self.viewport_mode = false; // Exit viewport mode when entering free mode
            self.paused = true; // Enforce pause in free mode
            self.free_line = cursor_row;
        }
        self.needs_render = true;
    }

    /// Exit current mode (viewport or free) or quit.
    ///
    /// Returns true if a mode was exited, false if should quit.
    pub fn exit_mode_or_quit(&mut self) -> bool {
        if self.viewport_mode {
            self.viewport_mode = false;
            self.needs_render = true;
            true
        } else if self.free_mode {
            self.free_mode = false;
            self.needs_render = true;
            true
        } else {
            false // Should quit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_correct_defaults() {
        let state = PlaybackState::new(80, 27);

        assert!(!state.paused);
        assert_eq!(state.speed, 1.0);
        assert_eq!(state.event_idx, 0);
        assert_eq!(state.current_time, 0.0);
        assert!(!state.show_help);
        assert!(!state.viewport_mode);
        assert!(!state.free_mode);
        assert_eq!(state.view_rows, 24); // 27 - 3 status lines
        assert_eq!(state.view_cols, 80);
        assert!(state.needs_render);
    }

    #[test]
    fn handle_resize_updates_dimensions() {
        let mut state = PlaybackState::new(80, 27);
        state.handle_resize(120, 40, 100, 50);

        assert_eq!(state.term_cols, 120);
        assert_eq!(state.term_rows, 40);
        assert_eq!(state.view_rows, 37); // 40 - 3
        assert_eq!(state.view_cols, 120);
    }

    #[test]
    fn handle_resize_clamps_offset() {
        let mut state = PlaybackState::new(80, 27);
        state.view_row_offset = 100;
        state.view_col_offset = 100;

        state.handle_resize(80, 27, 30, 30);

        // Offset should be clamped: 30 - 24 = 6 max row, 30 - 80 = 0 max col
        assert!(state.view_row_offset <= 6);
        assert_eq!(state.view_col_offset, 0);
    }

    #[test]
    fn toggle_pause_resets_timing() {
        let mut state = PlaybackState::new(80, 27);
        state.paused = true;
        state.current_time = 10.0;
        state.free_mode = true;

        state.toggle_pause();

        assert!(!state.paused);
        assert!(!state.free_mode); // Exited free mode
        assert_eq!(state.time_offset, 10.0); // Preserved current time
    }

    #[test]
    fn speed_up_increases_speed() {
        let mut state = PlaybackState::new(80, 27);
        state.speed_up();
        assert_eq!(state.speed, 1.5);
        state.speed_up();
        assert!((state.speed - 2.25).abs() < 0.01);
    }

    #[test]
    fn speed_up_maxes_at_16() {
        let mut state = PlaybackState::new(80, 27);
        state.speed = 15.0;
        state.speed_up();
        assert_eq!(state.speed, 16.0);
    }

    #[test]
    fn speed_down_decreases_speed() {
        let mut state = PlaybackState::new(80, 27);
        state.speed = 2.0;
        state.speed_down();
        assert!((state.speed - 1.333).abs() < 0.01);
    }

    #[test]
    fn speed_down_mins_at_0_1() {
        let mut state = PlaybackState::new(80, 27);
        state.speed = 0.15;
        state.speed_down();
        assert_eq!(state.speed, 0.1);
    }

    #[test]
    fn toggle_free_mode_enables_and_pauses() {
        let mut state = PlaybackState::new(80, 27);
        state.viewport_mode = true;

        state.toggle_free_mode(5);

        assert!(state.free_mode);
        assert!(state.paused);
        assert!(!state.viewport_mode);
        assert_eq!(state.free_line, 5);
    }

    #[test]
    fn toggle_viewport_mode_exits_free_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.free_mode = true;

        state.toggle_viewport_mode();

        assert!(state.viewport_mode);
        assert!(!state.free_mode);
    }

    #[test]
    fn exit_mode_exits_viewport_first() {
        let mut state = PlaybackState::new(80, 27);
        state.viewport_mode = true;

        assert!(state.exit_mode_or_quit()); // Should return true (mode exited)
        assert!(!state.viewport_mode);
    }

    #[test]
    fn exit_mode_exits_free_mode() {
        let mut state = PlaybackState::new(80, 27);
        state.free_mode = true;

        assert!(state.exit_mode_or_quit());
        assert!(!state.free_mode);
    }

    #[test]
    fn exit_mode_returns_false_when_no_mode() {
        let mut state = PlaybackState::new(80, 27);
        assert!(!state.exit_mode_or_quit()); // Should quit
    }

    #[test]
    fn input_result_enum_variants() {
        assert_eq!(InputResult::Continue, InputResult::Continue);
        assert_ne!(InputResult::Quit, InputResult::Continue);
        assert_ne!(InputResult::QuitWithFile, InputResult::Quit);
    }

    #[test]
    fn marker_position_stores_data() {
        let marker = MarkerPosition {
            time: 5.5,
            label: "Test marker".to_string(),
        };
        assert_eq!(marker.time, 5.5);
        assert_eq!(marker.label, "Test marker");
    }
}
