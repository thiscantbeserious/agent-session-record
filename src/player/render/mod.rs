//! Rendering components for the native player.
//!
//! This module contains all the UI rendering functions for the player,
//! including viewport, progress bar, status bar, help overlay, and scroll indicators.

mod ansi;
mod help;
mod progress;
mod scroll;
mod status;
mod viewport;

pub use ansi::{style_to_ansi_attrs, style_to_ansi_bg, style_to_ansi_fg};
pub use help::{calc_help_start_col, calc_help_start_row, render_help, HELP_BOX_WIDTH, HELP_LINES};
pub use progress::{build_progress_bar_chars, format_duration, render_progress_bar};
pub use scroll::{build_scroll_arrows, calc_scroll_directions, render_scroll_indicator};
pub use status::{count_digits, render_separator_line, render_status_bar};
pub use viewport::{render_single_line, render_viewport};
