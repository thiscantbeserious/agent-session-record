//! Scroll indicator rendering for the native player.
//!
//! Displays arrows indicating available scroll directions when
//! the recording is larger than the viewport.

use std::io;

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

/// Calculate which scroll directions are available.
///
/// # Arguments
/// * `row_offset` - Current vertical scroll offset
/// * `col_offset` - Current horizontal scroll offset
/// * `view_rows` - Number of visible rows
/// * `view_cols` - Number of visible columns
/// * `rec_rows` - Total recording height
/// * `rec_cols` - Total recording width
///
/// # Returns
/// A tuple of (can_up, can_down, can_left, can_right)
pub fn calc_scroll_directions(
    row_offset: usize,
    col_offset: usize,
    view_rows: usize,
    view_cols: usize,
    rec_rows: usize,
    rec_cols: usize,
) -> (bool, bool, bool, bool) {
    let can_up = row_offset > 0;
    let can_down = row_offset + view_rows < rec_rows;
    let can_left = col_offset > 0;
    let can_right = col_offset + view_cols < rec_cols;
    (can_up, can_down, can_left, can_right)
}

/// Build the scroll indicator arrow string.
///
/// # Arguments
/// * `can_up` - Whether scrolling up is possible
/// * `can_down` - Whether scrolling down is possible
/// * `can_left` - Whether scrolling left is possible
/// * `can_right` - Whether scrolling right is possible
///
/// # Returns
/// `None` if no scrolling is possible, otherwise `Some(arrow_string)`
pub fn build_scroll_arrows(
    can_up: bool,
    can_down: bool,
    can_left: bool,
    can_right: bool,
) -> Option<String> {
    if !can_up && !can_down && !can_left && !can_right {
        return None;
    }

    let mut arrows = Vec::new();
    if can_up {
        arrows.push("▲");
    }
    if can_down {
        arrows.push("▼");
    }
    if can_left {
        arrows.push("◀");
    }
    if can_right {
        arrows.push("▶");
    }

    if arrows.is_empty() {
        None
    } else {
        Some(arrows.join(" "))
    }
}

/// Render scroll indicator in top-right showing available scroll directions.
///
/// # Arguments
/// * `stdout` - The stdout handle to write to
/// * `term_cols` - Terminal width
/// * `row_offset` - Current vertical scroll offset
/// * `col_offset` - Current horizontal scroll offset
/// * `view_rows` - Number of visible rows
/// * `view_cols` - Number of visible columns
/// * `rec_rows` - Total recording height
/// * `rec_cols` - Total recording width
#[allow(clippy::too_many_arguments)]
pub fn render_scroll_indicator(
    stdout: &mut io::Stdout,
    term_cols: u16,
    row_offset: usize,
    col_offset: usize,
    view_rows: usize,
    view_cols: usize,
    rec_rows: usize,
    rec_cols: usize,
) -> Result<()> {
    let (can_up, can_down, can_left, can_right) = calc_scroll_directions(
        row_offset, col_offset, view_rows, view_cols, rec_rows, rec_cols,
    );

    let arrow_str = match build_scroll_arrows(can_up, can_down, can_left, can_right) {
        Some(s) => s,
        None => return Ok(()),
    };

    let arrows_count = [can_up, can_down, can_left, can_right]
        .iter()
        .filter(|&&x| x)
        .count();

    // Draw at top-right, completely aligned to edge
    let arrow_color = Color::Yellow;
    let bg_color = Color::AnsiValue(236); // Same as progress bar
                                          // Width = arrows + spaces between + padding on sides
    let display_width = (arrows_count * 2 + 1) as u16; // each arrow + space, plus padding
    let start_col = term_cols.saturating_sub(display_width);

    execute!(
        stdout,
        MoveTo(start_col, 0),
        SetBackgroundColor(bg_color),
        SetForegroundColor(arrow_color),
        Print(" "),
        Print(&arrow_str),
        Print(" "),
        ResetColor,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_scroll_when_viewport_fits() {
        let (up, down, left, right) = calc_scroll_directions(0, 0, 24, 80, 24, 80);
        assert!(!up);
        assert!(!down);
        assert!(!left);
        assert!(!right);
    }

    #[test]
    fn can_scroll_down_when_content_below() {
        let (up, down, left, right) = calc_scroll_directions(0, 0, 24, 80, 48, 80);
        assert!(!up);
        assert!(down);
        assert!(!left);
        assert!(!right);
    }

    #[test]
    fn can_scroll_up_when_offset_positive() {
        let (up, down, left, right) = calc_scroll_directions(10, 0, 24, 80, 48, 80);
        assert!(up);
        assert!(down); // Still more content below
        assert!(!left);
        assert!(!right);
    }

    #[test]
    fn can_scroll_right_when_content_wider() {
        let (up, down, left, right) = calc_scroll_directions(0, 0, 24, 80, 24, 120);
        assert!(!up);
        assert!(!down);
        assert!(!left);
        assert!(right);
    }

    #[test]
    fn can_scroll_left_when_col_offset() {
        let (up, down, left, right) = calc_scroll_directions(0, 20, 24, 80, 24, 120);
        assert!(!up);
        assert!(!down);
        assert!(left);
        assert!(right);
    }

    #[test]
    fn all_directions_when_in_middle() {
        // Viewport in middle of larger content
        let (up, down, left, right) = calc_scroll_directions(10, 10, 24, 80, 48, 160);
        assert!(up);
        assert!(down);
        assert!(left);
        assert!(right);
    }

    #[test]
    fn at_bottom_right_corner() {
        // At bottom-right, can only scroll up and left
        let (up, down, left, right) = calc_scroll_directions(24, 40, 24, 80, 48, 120);
        assert!(up);
        assert!(!down); // At bottom
        assert!(left);
        assert!(!right); // At right edge
    }

    #[test]
    fn no_arrows_when_no_scroll() {
        let result = build_scroll_arrows(false, false, false, false);
        assert!(result.is_none());
    }

    #[test]
    fn up_arrow_only() {
        let result = build_scroll_arrows(true, false, false, false);
        assert_eq!(result, Some("▲".to_string()));
    }

    #[test]
    fn down_arrow_only() {
        let result = build_scroll_arrows(false, true, false, false);
        assert_eq!(result, Some("▼".to_string()));
    }

    #[test]
    fn left_arrow_only() {
        let result = build_scroll_arrows(false, false, true, false);
        assert_eq!(result, Some("◀".to_string()));
    }

    #[test]
    fn right_arrow_only() {
        let result = build_scroll_arrows(false, false, false, true);
        assert_eq!(result, Some("▶".to_string()));
    }

    #[test]
    fn up_and_down_arrows() {
        let result = build_scroll_arrows(true, true, false, false);
        assert_eq!(result, Some("▲ ▼".to_string()));
    }

    #[test]
    fn all_arrows() {
        let result = build_scroll_arrows(true, true, true, true);
        assert_eq!(result, Some("▲ ▼ ◀ ▶".to_string()));
    }

    #[test]
    fn horizontal_arrows_only() {
        let result = build_scroll_arrows(false, false, true, true);
        assert_eq!(result, Some("◀ ▶".to_string()));
    }
}
