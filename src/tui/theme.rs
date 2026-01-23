//! Theme configuration for TUI
//!
//! Centralizes all color and style definitions for easy customization.

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
    /// AGR theme - gray text with green logo accent.
    pub fn claude_code() -> Self {
        Self {
            text_primary: Color::Rgb(180, 180, 180), // Light gray for help text
            text_secondary: Color::Rgb(100, 100, 100), // Dim gray for footer hints
            accent: Color::Rgb(50, 205, 50),         // Lime green (vibrant) for logo
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
        // text_primary is light gray, accent is lime green
        assert_eq!(theme.text_primary, Color::Rgb(180, 180, 180));
        assert_eq!(theme.accent, Color::Rgb(50, 205, 50));
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
        assert_eq!(theme.text_style().fg, Some(Color::Rgb(180, 180, 180)));
        assert_eq!(
            theme.text_secondary_style().fg,
            Some(Color::Rgb(100, 100, 100))
        );
        assert_eq!(theme.accent_style().fg, Some(Color::Rgb(50, 205, 50)));
    }
}
