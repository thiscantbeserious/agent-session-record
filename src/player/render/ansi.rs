//! ANSI escape code conversion utilities.
//!
//! Converts terminal cell styles to ANSI escape sequences for rendering.

use crate::terminal::{CellStyle, Color as TermColor};

/// Convert cell style foreground to ANSI escape code.
///
/// Appends the appropriate ANSI escape sequence to the buffer
/// for the foreground color.
///
/// # Arguments
/// * `style` - The cell style to convert
/// * `buf` - The output buffer to append to
///
/// # Returns
/// `true` if a code was appended, `false` if the color is default
pub fn style_to_ansi_fg(style: &CellStyle, buf: &mut String) -> bool {
    match &style.fg {
        TermColor::Default => false,
        TermColor::Black => {
            buf.push_str("\x1b[30m");
            true
        }
        TermColor::Red => {
            buf.push_str("\x1b[31m");
            true
        }
        TermColor::Green => {
            buf.push_str("\x1b[32m");
            true
        }
        TermColor::Yellow => {
            buf.push_str("\x1b[33m");
            true
        }
        TermColor::Blue => {
            buf.push_str("\x1b[34m");
            true
        }
        TermColor::Magenta => {
            buf.push_str("\x1b[35m");
            true
        }
        TermColor::Cyan => {
            buf.push_str("\x1b[36m");
            true
        }
        TermColor::White => {
            buf.push_str("\x1b[37m");
            true
        }
        TermColor::BrightBlack => {
            buf.push_str("\x1b[90m");
            true
        }
        TermColor::BrightRed => {
            buf.push_str("\x1b[91m");
            true
        }
        TermColor::BrightGreen => {
            buf.push_str("\x1b[92m");
            true
        }
        TermColor::BrightYellow => {
            buf.push_str("\x1b[93m");
            true
        }
        TermColor::BrightBlue => {
            buf.push_str("\x1b[94m");
            true
        }
        TermColor::BrightMagenta => {
            buf.push_str("\x1b[95m");
            true
        }
        TermColor::BrightCyan => {
            buf.push_str("\x1b[96m");
            true
        }
        TermColor::BrightWhite => {
            buf.push_str("\x1b[97m");
            true
        }
        TermColor::Indexed(n) => {
            buf.push_str("\x1b[38;5;");
            buf.push_str(&n.to_string());
            buf.push('m');
            true
        }
        TermColor::Rgb(r, g, b) => {
            buf.push_str("\x1b[38;2;");
            buf.push_str(&r.to_string());
            buf.push(';');
            buf.push_str(&g.to_string());
            buf.push(';');
            buf.push_str(&b.to_string());
            buf.push('m');
            true
        }
    }
}

/// Convert cell style background to ANSI escape code.
///
/// Appends the appropriate ANSI escape sequence to the buffer
/// for the background color.
///
/// # Arguments
/// * `style` - The cell style to convert
/// * `buf` - The output buffer to append to
///
/// # Returns
/// `true` if a code was appended, `false` if the color is default
pub fn style_to_ansi_bg(style: &CellStyle, buf: &mut String) -> bool {
    match &style.bg {
        TermColor::Default => false,
        TermColor::Black => {
            buf.push_str("\x1b[40m");
            true
        }
        TermColor::Red => {
            buf.push_str("\x1b[41m");
            true
        }
        TermColor::Green => {
            buf.push_str("\x1b[42m");
            true
        }
        TermColor::Yellow => {
            buf.push_str("\x1b[43m");
            true
        }
        TermColor::Blue => {
            buf.push_str("\x1b[44m");
            true
        }
        TermColor::Magenta => {
            buf.push_str("\x1b[45m");
            true
        }
        TermColor::Cyan => {
            buf.push_str("\x1b[46m");
            true
        }
        TermColor::White => {
            buf.push_str("\x1b[47m");
            true
        }
        TermColor::BrightBlack => {
            buf.push_str("\x1b[100m");
            true
        }
        TermColor::BrightRed => {
            buf.push_str("\x1b[101m");
            true
        }
        TermColor::BrightGreen => {
            buf.push_str("\x1b[102m");
            true
        }
        TermColor::BrightYellow => {
            buf.push_str("\x1b[103m");
            true
        }
        TermColor::BrightBlue => {
            buf.push_str("\x1b[104m");
            true
        }
        TermColor::BrightMagenta => {
            buf.push_str("\x1b[105m");
            true
        }
        TermColor::BrightCyan => {
            buf.push_str("\x1b[106m");
            true
        }
        TermColor::BrightWhite => {
            buf.push_str("\x1b[107m");
            true
        }
        TermColor::Indexed(n) => {
            buf.push_str("\x1b[48;5;");
            buf.push_str(&n.to_string());
            buf.push('m');
            true
        }
        TermColor::Rgb(r, g, b) => {
            buf.push_str("\x1b[48;2;");
            buf.push_str(&r.to_string());
            buf.push(';');
            buf.push_str(&g.to_string());
            buf.push(';');
            buf.push_str(&b.to_string());
            buf.push('m');
            true
        }
    }
}

/// Append ANSI codes for text attributes (bold, dim, italic, underline, reverse).
///
/// # Arguments
/// * `style` - The cell style to convert
/// * `buf` - The output buffer to append to
pub fn style_to_ansi_attrs(style: &CellStyle, buf: &mut String) {
    if style.bold {
        buf.push_str("\x1b[1m");
    }
    if style.dim {
        buf.push_str("\x1b[2m");
    }
    if style.italic {
        buf.push_str("\x1b[3m");
    }
    if style.underline {
        buf.push_str("\x1b[4m");
    }
    if style.reverse {
        buf.push_str("\x1b[7m");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_to_ansi_fg_default_returns_false() {
        let style = CellStyle::default();
        let mut buf = String::new();
        assert!(!style_to_ansi_fg(&style, &mut buf));
        assert!(buf.is_empty());
    }

    #[test]
    fn style_to_ansi_fg_red_appends_code() {
        let style = CellStyle {
            fg: TermColor::Red,
            ..Default::default()
        };
        let mut buf = String::new();
        assert!(style_to_ansi_fg(&style, &mut buf));
        assert_eq!(buf, "\x1b[31m");
    }

    #[test]
    fn style_to_ansi_fg_all_basic_colors() {
        let test_cases = [
            (TermColor::Black, "\x1b[30m"),
            (TermColor::Red, "\x1b[31m"),
            (TermColor::Green, "\x1b[32m"),
            (TermColor::Yellow, "\x1b[33m"),
            (TermColor::Blue, "\x1b[34m"),
            (TermColor::Magenta, "\x1b[35m"),
            (TermColor::Cyan, "\x1b[36m"),
            (TermColor::White, "\x1b[37m"),
        ];

        for (color, expected) in test_cases {
            let style = CellStyle {
                fg: color,
                ..Default::default()
            };
            let mut buf = String::new();
            assert!(style_to_ansi_fg(&style, &mut buf));
            assert_eq!(buf, expected, "Failed for {:?}", color);
        }
    }

    #[test]
    fn style_to_ansi_fg_all_bright_colors() {
        let test_cases = [
            (TermColor::BrightBlack, "\x1b[90m"),
            (TermColor::BrightRed, "\x1b[91m"),
            (TermColor::BrightGreen, "\x1b[92m"),
            (TermColor::BrightYellow, "\x1b[93m"),
            (TermColor::BrightBlue, "\x1b[94m"),
            (TermColor::BrightMagenta, "\x1b[95m"),
            (TermColor::BrightCyan, "\x1b[96m"),
            (TermColor::BrightWhite, "\x1b[97m"),
        ];

        for (color, expected) in test_cases {
            let style = CellStyle {
                fg: color,
                ..Default::default()
            };
            let mut buf = String::new();
            assert!(style_to_ansi_fg(&style, &mut buf));
            assert_eq!(buf, expected, "Failed for {:?}", color);
        }
    }

    #[test]
    fn style_to_ansi_fg_indexed_color() {
        let style = CellStyle {
            fg: TermColor::Indexed(196),
            ..Default::default()
        };
        let mut buf = String::new();
        assert!(style_to_ansi_fg(&style, &mut buf));
        assert_eq!(buf, "\x1b[38;5;196m");
    }

    #[test]
    fn style_to_ansi_fg_rgb_color() {
        let style = CellStyle {
            fg: TermColor::Rgb(255, 128, 64),
            ..Default::default()
        };
        let mut buf = String::new();
        assert!(style_to_ansi_fg(&style, &mut buf));
        assert_eq!(buf, "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn style_to_ansi_bg_default_returns_false() {
        let style = CellStyle::default();
        let mut buf = String::new();
        assert!(!style_to_ansi_bg(&style, &mut buf));
        assert!(buf.is_empty());
    }

    #[test]
    fn style_to_ansi_bg_all_basic_colors() {
        let test_cases = [
            (TermColor::Black, "\x1b[40m"),
            (TermColor::Red, "\x1b[41m"),
            (TermColor::Green, "\x1b[42m"),
            (TermColor::Yellow, "\x1b[43m"),
            (TermColor::Blue, "\x1b[44m"),
            (TermColor::Magenta, "\x1b[45m"),
            (TermColor::Cyan, "\x1b[46m"),
            (TermColor::White, "\x1b[47m"),
        ];

        for (color, expected) in test_cases {
            let style = CellStyle {
                bg: color,
                ..Default::default()
            };
            let mut buf = String::new();
            assert!(style_to_ansi_bg(&style, &mut buf));
            assert_eq!(buf, expected, "Failed for {:?}", color);
        }
    }

    #[test]
    fn style_to_ansi_bg_indexed_color() {
        let style = CellStyle {
            bg: TermColor::Indexed(236),
            ..Default::default()
        };
        let mut buf = String::new();
        assert!(style_to_ansi_bg(&style, &mut buf));
        assert_eq!(buf, "\x1b[48;5;236m");
    }

    #[test]
    fn style_to_ansi_bg_rgb_color() {
        let style = CellStyle {
            bg: TermColor::Rgb(0, 128, 255),
            ..Default::default()
        };
        let mut buf = String::new();
        assert!(style_to_ansi_bg(&style, &mut buf));
        assert_eq!(buf, "\x1b[48;2;0;128;255m");
    }

    #[test]
    fn style_to_ansi_attrs_bold() {
        let style = CellStyle {
            bold: true,
            ..Default::default()
        };
        let mut buf = String::new();
        style_to_ansi_attrs(&style, &mut buf);
        assert_eq!(buf, "\x1b[1m");
    }

    #[test]
    fn style_to_ansi_attrs_multiple() {
        let style = CellStyle {
            bold: true,
            italic: true,
            underline: true,
            ..Default::default()
        };
        let mut buf = String::new();
        style_to_ansi_attrs(&style, &mut buf);
        assert_eq!(buf, "\x1b[1m\x1b[3m\x1b[4m");
    }
}
