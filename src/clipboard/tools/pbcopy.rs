//! macOS pbcopy clipboard tool.

use crate::clipboard::result::CopyMethod;
use crate::clipboard::tool::{CopyTool, CopyToolError};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// macOS pasteboard copy tool.
///
/// Uses `pbcopy` to copy text content to the clipboard.
/// Does not support file copy - use OsaScript for that.
pub struct Pbcopy;

impl Pbcopy {
    /// Create a new Pbcopy tool.
    pub fn new() -> Self {
        Self
    }
}

impl CopyTool for Pbcopy {
    fn method(&self) -> CopyMethod {
        CopyMethod::Pbcopy
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn can_copy_files(&self) -> bool {
        false
    }

    fn try_copy_file(&self, _path: &Path) -> Result<(), CopyToolError> {
        Err(CopyToolError::NotSupported)
    }

    fn try_copy_text(&self, text: &str) -> Result<(), CopyToolError> {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CopyToolError::Failed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| CopyToolError::Failed(e.to_string()))?;
        }

        let status = child
            .wait()
            .map_err(|e| CopyToolError::Failed(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(CopyToolError::Failed("pbcopy failed".to_string()))
        }
    }
}

impl Default for Pbcopy {
    fn default() -> Self {
        Self::new()
    }
}
