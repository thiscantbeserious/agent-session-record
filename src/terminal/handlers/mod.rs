//! Terminal escape sequence handlers.
//!
//! Organized by category:
//! - cursor: Cursor movement and positioning
//! - scroll: Scroll region and scrolling operations
//! - editing: Erase and delete operations
//! - style: SGR (Select Graphic Rendition) handling

pub mod cursor;
pub mod editing;
pub mod scroll;
pub mod style;

// TODO: Stage 8 - Add observability functions for unhandled sequences
