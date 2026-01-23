//! Theme configuration for TUI and CLI
//!
//! Centralizes all color and style definitions for easy customization.
//! Provides both ratatui styles (for TUI) and ANSI escape codes (for CLI).

use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the TUI.
///
/// All colors and styles are defined here for easy customization.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary text color (used for most content)
    pub text_primary: Color,
    /// Secondary/dimmed text color
    pub text_secondary: Color,
    /// Accent color for highlights and important elements
    pub accent: Color,
    /// Error/warning color
    pub error: Color,
    /// Success color
    pub success: Color,
    /// Background color (usually default/transparent)
    pub background: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::claude_code()
    }
}

impl Theme {
    /// AGR theme - light gray text with green logo accent.
    /// Uses standard ANSI colors for consistent terminal rendering.
    pub fn claude_code() -> Self {
        Self {
            text_primary: Color::Gray,       // Light gray for help text
            text_secondary: Color::DarkGray, // Dark gray for footer hints
            accent: Color::Green,            // Standard ANSI green for logo (matches terminal)
            error: Color::Red,
            success: Color::Green,
            background: Color::Reset,
        }
    }

    /// Classic terminal theme - white text.
    pub fn classic() -> Self {
        Self {
            text_primary: Color::White,
            text_secondary: Color::DarkGray,
            accent: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            background: Color::Reset,
        }
    }

    /// Cyan/blue theme.
    pub fn ocean() -> Self {
        Self {
            text_primary: Color::Cyan,
            text_secondary: Color::DarkGray,
            accent: Color::LightCyan,
            error: Color::Red,
            success: Color::Green,
            background: Color::Reset,
        }
    }

    // Style helpers

    /// Style for primary text content.
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    /// Style for secondary/dimmed text.
    pub fn text_secondary_style(&self) -> Style {
        Style::default().fg(self.text_secondary)
    }

    /// Style for accented/highlighted text.
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Style for bold accented text (keybindings, etc).
    pub fn accent_bold_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for error text.
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Style for success text.
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    // ANSI color helpers for CLI output

    /// Format text with the accent color (for CLI output).
    pub fn accent_text(&self, text: &str) -> String {
        format!("{}{}{}", color_to_ansi(self.accent), text, ANSI_RESET)
    }

    /// Format text with the primary color (for CLI output).
    pub fn primary_text(&self, text: &str) -> String {
        format!("{}{}{}", color_to_ansi(self.text_primary), text, ANSI_RESET)
    }

    /// Format text with the secondary color (for CLI output).
    pub fn secondary_text(&self, text: &str) -> String {
        format!(
            "{}{}{}",
            color_to_ansi(self.text_secondary),
            text,
            ANSI_RESET
        )
    }

    /// Format text with the error color (for CLI output).
    pub fn error_text(&self, text: &str) -> String {
        format!("{}{}{}", color_to_ansi(self.error), text, ANSI_RESET)
    }

    /// Format text with the success color (for CLI output).
    pub fn success_text(&self, text: &str) -> String {
        format!("{}{}{}", color_to_ansi(self.success), text, ANSI_RESET)
    }
}

/// ANSI reset sequence
const ANSI_RESET: &str = "\x1b[0m";

/// Convert a ratatui Color to an ANSI escape code.
fn color_to_ansi(color: Color) -> &'static str {
    match color {
        Color::Black => "\x1b[30m",
        Color::Red => "\x1b[31m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Blue => "\x1b[34m",
        Color::Magenta => "\x1b[35m",
        Color::Cyan => "\x1b[36m",
        Color::Gray => "\x1b[37m",
        Color::DarkGray => "\x1b[90m",
        Color::LightRed => "\x1b[91m",
        Color::LightGreen => "\x1b[92m",
        Color::LightYellow => "\x1b[93m",
        Color::LightBlue => "\x1b[94m",
        Color::LightMagenta => "\x1b[95m",
        Color::LightCyan => "\x1b[96m",
        Color::White => "\x1b[97m",
        Color::Reset => "\x1b[0m",
        // For RGB and indexed colors, fall back to reset (no color)
        _ => "",
    }
}

/// Global theme instance.
///
/// In the future, this could be loaded from config.
pub fn current_theme() -> Theme {
    Theme::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_claude_code() {
        let theme = Theme::default();
        // text_primary is light gray, accent is standard ANSI green
        assert_eq!(theme.text_primary, Color::Gray);
        assert_eq!(theme.accent, Color::Green);
    }

    #[test]
    fn classic_theme_uses_white() {
        let theme = Theme::classic();
        assert_eq!(theme.text_primary, Color::White);
    }

    #[test]
    fn ocean_theme_uses_cyan() {
        let theme = Theme::ocean();
        assert_eq!(theme.text_primary, Color::Cyan);
    }

    #[test]
    fn style_helpers_return_correct_colors() {
        let theme = Theme::claude_code();
        assert_eq!(theme.text_style().fg, Some(Color::Gray));
        assert_eq!(theme.text_secondary_style().fg, Some(Color::DarkGray));
        assert_eq!(theme.accent_style().fg, Some(Color::Green));
    }

    #[test]
    fn ansi_text_helpers_wrap_with_color_codes() {
        let theme = Theme::claude_code();

        // Accent text should wrap with green
        let accent = theme.accent_text("test");
        assert!(accent.starts_with("\x1b[32m")); // Green
        assert!(accent.ends_with("\x1b[0m")); // Reset
        assert!(accent.contains("test"));

        // Primary text should wrap with gray
        let primary = theme.primary_text("hello");
        assert!(primary.starts_with("\x1b[37m")); // Gray
        assert!(primary.ends_with("\x1b[0m"));
        assert!(primary.contains("hello"));
    }

    #[test]
    fn color_to_ansi_maps_standard_colors() {
        assert_eq!(color_to_ansi(Color::Green), "\x1b[32m");
        assert_eq!(color_to_ansi(Color::Red), "\x1b[31m");
        assert_eq!(color_to_ansi(Color::Gray), "\x1b[37m");
        assert_eq!(color_to_ansi(Color::DarkGray), "\x1b[90m");
        assert_eq!(color_to_ansi(Color::Reset), "\x1b[0m");
    }
}
