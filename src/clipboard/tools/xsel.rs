//! Linux xsel clipboard tool.

use crate::clipboard::result::CopyMethod;
use crate::clipboard::tool::{CopyTool, CopyToolError};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Linux X11 clipboard tool using xsel.
///
/// Uses `xsel` to copy text content to the clipboard.
/// Does not support file copy - use xclip for that.
pub struct Xsel;

impl Xsel {
    /// Create a new Xsel tool.
    pub fn new() -> Self {
        Self
    }

    /// Check if xsel is installed.
    fn tool_exists() -> bool {
        Command::new("which")
            .arg("xsel")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl CopyTool for Xsel {
    fn method(&self) -> CopyMethod {
        CopyMethod::Xsel
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
        let mut child = Command::new("xsel")
            .args(["--clipboard", "--input"])
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
            Err(CopyToolError::Failed("xsel failed".to_string()))
        }
    }
}

impl Default for Xsel {
    fn default() -> Self {
        Self::new()
    }
}
