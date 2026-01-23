//! UI rendering helpers for TUI
//!
//! Common UI utilities and layout helpers.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::widgets::Logo;

/// Render the logo centered at the top of the frame.
pub fn render_logo(frame: &mut Frame) {
    let area = frame.area();
    let logo = Logo::new();
    frame.render_widget(logo, area);
}

/// Create a centered layout with the given constraints.
///
/// Returns the center area that can be used for content.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_rect_creates_smaller_area() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        // Centered area should be roughly 50% of original
        assert!(centered.width <= 55); // Allow some rounding
        assert!(centered.height <= 55);
    }

    #[test]
    fn centered_rect_is_centered() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, area);

        // Should be roughly centered
        assert!(centered.x >= 20 && centered.x <= 30);
        assert!(centered.y >= 20 && centered.y <= 30);
    }
}
