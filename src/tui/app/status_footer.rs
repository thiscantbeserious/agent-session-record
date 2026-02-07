//! Shared status line and footer rendering for TUI explorer applications
//!
//! Provides rendering functions for the status bar (filter info, mode prompts)
//! and the footer bar (keybinding hints).

use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::theme::current_theme;

/// Render a status line with the given text.
///
/// Displays the text in the secondary text color of the current theme.
/// Each app computes its own mode-aware status text and passes it here.
pub fn render_status_line(frame: &mut Frame, area: Rect, text: &str) {
    let theme = current_theme();
    let status = Paragraph::new(text.to_string()).style(Style::default().fg(theme.text_secondary));
    frame.render_widget(status, area);
}

/// Render a centered footer with keybinding hints.
///
/// Takes pairs of (key, description) and joins them with " | " separators.
/// Each app passes its own mode-specific key hints.
///
/// Example: `&[("q", "quit"), ("?", "help")]` renders as `"q: quit | ?: help"`.
#[allow(dead_code)]
pub fn render_footer(frame: &mut Frame, area: Rect, keys: &[(&str, &str)]) {
    let theme = current_theme();
    let spans: Vec<Span<'static>> = build_footer_spans(keys, &theme);
    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(theme.text_secondary))
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

/// Build styled spans for footer keybinding hints.
///
/// Each key is highlighted with the theme accent color, descriptions use
/// the secondary text color, and entries are separated by " | ".
fn build_footer_spans(keys: &[(&str, &str)], theme: &crate::theme::Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::with_capacity(keys.len() * 3);
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                " | ".to_string(),
                Style::default().fg(theme.text_secondary),
            ));
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(theme.accent),
        ));
        spans.push(Span::styled(
            format!(": {}", desc),
            Style::default().fg(theme.text_secondary),
        ));
    }
    spans
}

/// Render a centered footer from a pre-formatted text string.
///
/// Simpler alternative to `render_footer` when the footer text is already
/// composed (e.g., from a mode-specific string literal).
pub fn render_footer_text(frame: &mut Frame, area: Rect, text: &str) {
    let theme = current_theme();
    let footer = Paragraph::new(text.to_string())
        .style(Style::default().fg(theme.text_secondary))
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}
