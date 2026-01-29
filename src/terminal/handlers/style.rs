//! SGR (Select Graphic Rendition) handler.
//!
//! Handles CSI m sequence for text styling:
//! - Reset (0)
//! - Bold, dim, italic, underline, reverse (1-7)
//! - Standard foreground/background colors (30-47)
//! - Extended colors - 256-color mode (38;5;n, 48;5;n)
//! - Extended colors - RGB mode (38;2;r;g;b, 48;2;r;g;b)
//! - Bright foreground/background colors (90-107)

// TODO: Stage 7 - Move SGR handler here
