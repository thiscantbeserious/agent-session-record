//! Clipboard operation errors.

use std::path::PathBuf;

/// Errors that can occur during clipboard operations.
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("No clipboard tool available. On Linux, install xclip, xsel, or wl-copy.")]
    NoToolAvailable,

    #[error("Clipboard tool '{tool}' failed: {message}")]
    ToolFailed { tool: &'static str, message: String },

    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Platform not supported (only macOS and Linux)")]
    UnsupportedPlatform,
}
