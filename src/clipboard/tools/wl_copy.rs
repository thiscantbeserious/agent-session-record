//! Linux Wayland wl-copy clipboard tool.

use crate::clipboard::result::CopyMethod;
use crate::clipboard::tool::{CopyTool, CopyToolError};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Linux Wayland clipboard tool using wl-copy.
///
/// Uses `wl-copy` to copy text content to the clipboard.
/// Does not support file copy for our use case.
pub struct WlCopy;

impl WlCopy {
    /// Create a new WlCopy tool.
    pub fn new() -> Self {
        Self
    }

    /// Check if wl-copy is installed.
    fn tool_exists() -> bool {
        Command::new("which")
            .arg("wl-copy")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl CopyTool for WlCopy {
    fn method(&self) -> CopyMethod {
        CopyMethod::WlCopy
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux") && Self::tool_exists()
    }

    fn can_copy_files(&self) -> bool {
        false
    }

    fn try_copy_file(&self, _path: &Path) -> Result<(), CopyToolError> {
        Err(CopyToolError::NotSupported)
    }

    fn try_copy_text(&self, text: &str) -> Result<(), CopyToolError> {
        let mut child = Command::new("wl-copy")
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
            Err(CopyToolError::Failed("wl-copy failed".to_string()))
        }
    }
}

impl Default for WlCopy {
    fn default() -> Self {
        Self::new()
    }
}
