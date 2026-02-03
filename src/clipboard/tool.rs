//! CopyTool trait and related error types.

use super::result::CopyMethod;
use std::path::Path;

/// A tool that can copy content to the system clipboard.
///
/// Each implementation wraps a specific OS tool (osascript, xclip, etc.)
/// and knows how to invoke it correctly.
pub trait CopyTool: Send + Sync {
    /// The method identifier for this tool.
    fn method(&self) -> CopyMethod;

    /// Human-readable name for error messages.
    fn name(&self) -> &'static str {
        self.method().name()
    }

    /// Check if this tool is available on the system.
    ///
    /// Should be fast - typically checks if the binary exists.
    fn is_available(&self) -> bool;

    /// Whether this tool supports copying files as file references.
    ///
    /// If false, only `try_copy_text` will be called.
    fn can_copy_files(&self) -> bool;

    /// Try to copy a file as a file reference.
    ///
    /// The file at `path` should be copyable to apps that accept file drops.
    fn try_copy_file(&self, path: &Path) -> Result<(), CopyToolError>;

    /// Try to copy text content to the clipboard.
    fn try_copy_text(&self, text: &str) -> Result<(), CopyToolError>;
}

/// Error from a specific tool operation.
#[derive(Debug, Clone)]
pub enum CopyToolError {
    /// Tool doesn't support this operation
    NotSupported,
    /// Tool execution failed
    Failed(String),
    /// Tool not found on system
    NotFound,
}
