//! Help overlay rendering for the native player.
//!
//! Displays a centered help overlay with all available keyboard shortcuts.

use std::io;

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

/// Help text lines for the help overlay.
pub const HELP_LINES: &[&str] = &[
    "",
    "  ╔═══════════════════════════════════════════╗",
    "  ║          AGR Native Player Help           ║",
    "  ╠═══════════════════════════════════════════╣",
    "  ║                                           ║",
    "  ║  Playback                                 ║",
    "  ║    Space      Pause / Resume              ║",
    "  ║    <-/->      Seek +/-5s                  ║",
    "  ║    Shift+<-/->  Seek +/-5%                ║",
    "  ║    +/-        Speed up / down             ║",
    "  ║    Home/End   Go to start / end           ║",
    "  ║                                           ║",
    "  ║  Markers                                  ║",
    "  ║    m          Jump to next marker         ║",
    "  ║                                           ║",
    "  ║  Free Mode (line-by-line navigation)      ║",
    "  ║    f          Toggle free mode            ║",
    "  ║    Up/Down    Move highlight up/down      ║",
    "  ║    Esc        Exit free mode              ║",
    "  ║                                           ║",
    "  ║  Viewport                                 ║",
    "  ║    v          Toggle viewport mode        ║",
    "  ║    Up/Down/L/R Scroll viewport (v mode)   ║",
    "  ║    r          Resize to recording         ║",
    "  ║    Esc        Exit viewport mode          ║",
    "  ║                                           ║",
    "  ║  General                                  ║",
    "  ║    ?          Show this help              ║",
    "  ║    q          Quit player                 ║",
    "  ║                                           ║",
    "  ║         Press any key to close            ║",
    "  ╚═══════════════════════════════════════════╝",
    "",
];

/// Width of the help box (for centering calculations).
pub const HELP_BOX_WIDTH: usize = 47;

/// Calculate the starting row for centering the help box.
///
/// # Arguments
/// * `term_height` - Terminal height in rows
///
/// # Returns
/// The row number to start rendering the help box at
pub fn calc_help_start_row(term_height: u16) -> u16 {
    let box_height = HELP_LINES.len() as u16;
    (term_height.saturating_sub(box_height)) / 2
}

/// Calculate the starting column for centering the help box.
///
/// # Arguments
/// * `term_width` - Terminal width in columns
///
/// # Returns
/// The column number to start rendering the help box at
pub fn calc_help_start_col(term_width: u16) -> u16 {
    ((term_width as usize).saturating_sub(HELP_BOX_WIDTH) / 2) as u16
}

/// Render the help overlay.
///
/// Clears the screen and draws a centered help box with all shortcuts.
///
/// # Arguments
/// * `stdout` - The stdout handle to write to
/// * `width` - Terminal width
/// * `height` - Terminal height
pub fn render_help(stdout: &mut io::Stdout, width: u16, height: u16) -> Result<()> {
    let start_row = calc_help_start_row(height);
    let col = calc_help_start_col(width);

    execute!(stdout, Clear(ClearType::All))?;

    for (i, line) in HELP_LINES.iter().enumerate() {
        let row = start_row + i as u16;
        execute!(
            stdout,
            MoveTo(col, row),
            SetForegroundColor(Color::Green),
            Print(line),
            ResetColor,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_lines_not_empty() {
        assert!(!HELP_LINES.is_empty());
    }

    #[test]
    fn help_lines_has_title() {
        let has_title = HELP_LINES
            .iter()
            .any(|line| line.contains("AGR Native Player Help"));
        assert!(has_title);
    }

    #[test]
    fn help_lines_has_quit_instruction() {
        let has_quit = HELP_LINES
            .iter()
            .any(|line| line.contains("q") && line.contains("Quit"));
        assert!(has_quit);
    }

    #[test]
    fn help_lines_has_close_instruction() {
        let has_close = HELP_LINES
            .iter()
            .any(|line| line.contains("Press any key to close"));
        assert!(has_close);
    }

    #[test]
    fn help_box_width_is_correct() {
        assert_eq!(HELP_BOX_WIDTH, 47);
    }

    #[test]
    fn calc_help_start_row_centers_vertically() {
        let start = calc_help_start_row(100);
        let box_height = HELP_LINES.len() as u16;
        assert_eq!(start, (100 - box_height) / 2);
    }

    #[test]
    fn calc_help_start_row_handles_small_terminal() {
        let start = calc_help_start_row(10);
        assert_eq!(start, 0); // saturating_sub prevents underflow
    }

    #[test]
    fn calc_help_start_col_centers_horizontally() {
        let col = calc_help_start_col(120);
        // (120 - 47) / 2 = 36
        assert_eq!(col, 36);
    }

    #[test]
    fn calc_help_start_col_handles_narrow_terminal() {
        let col = calc_help_start_col(40);
        // (40 - 47) saturating_sub = 0, / 2 = 0
        assert_eq!(col, 0);
    }
}
