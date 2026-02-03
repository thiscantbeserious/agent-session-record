//! Copy orchestrator for clipboard operations.

use super::error::ClipboardError;
use super::result::CopyResult;
use super::tool::{CopyTool, CopyToolError};
use super::tools::platform_tools;
use std::path::Path;

/// Orchestrates clipboard copy operations using available tools.
///
/// Tries tools in priority order:
/// 1. File copy with tools that support it
/// 2. Content copy as fallback
pub struct Copy {
    tools: Vec<Box<dyn CopyTool>>,
}

impl Copy {
    /// Create with platform-appropriate tools.
    pub fn new() -> Self {
        Self {
            tools: platform_tools(),
        }
    }

    /// Create with specific tools (for testing).
    pub fn with_tools(tools: Vec<Box<dyn CopyTool>>) -> Self {
        Self { tools }
    }

    /// Get a reference to the tools list.
    pub fn tools(&self) -> &[Box<dyn CopyTool>] {
        &self.tools
    }

    /// Copy a file to the clipboard.
    ///
    /// Tries file copy first, falls back to content copy.
    pub fn file(&self, path: &Path) -> Result<CopyResult, ClipboardError> {
        // Validate file exists
        if !path.exists() {
            return Err(ClipboardError::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        // Try file copy with tools that support it
        for tool in &self.tools {
            if tool.is_available() && tool.can_copy_files() {
                match tool.try_copy_file(path) {
                    Ok(()) => {
                        return Ok(CopyResult::file_copied(tool.method()));
                    }
                    Err(CopyToolError::NotSupported) => continue,
                    Err(CopyToolError::NotFound) => continue,
                    Err(CopyToolError::Failed(_)) => continue, // Try next tool
                }
            }
        }

        // Fall back to content copy
        let content = std::fs::read_to_string(path)?;
        let size = content.len();

        for tool in &self.tools {
            if tool.is_available() {
                match tool.try_copy_text(&content) {
                    Ok(()) => {
                        return Ok(CopyResult::content_copied(tool.method(), size));
                    }
                    Err(CopyToolError::NotSupported) => continue,
                    Err(CopyToolError::NotFound) => continue,
                    Err(CopyToolError::Failed(_)) => continue,
                }
            }
        }

        Err(ClipboardError::NoToolAvailable)
    }
}

impl Default for Copy {
    fn default() -> Self {
        Self::new()
    }
}
