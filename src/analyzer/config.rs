//! Configuration for the content extraction pipeline.

/// Configuration for the content extraction pipeline.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Strip ANSI escape sequences (always true)
    pub strip_ansi: bool,
    /// Strip control characters (always true)
    pub strip_control_chars: bool,
    /// Deduplicate progress lines using \r
    pub dedupe_progress_lines: bool,
    /// Normalize excessive whitespace
    pub normalize_whitespace: bool,
    /// Maximum consecutive newlines allowed
    pub max_consecutive_newlines: usize,
    /// Strip box drawing characters
    pub strip_box_drawing: bool,
    /// Strip spinner animation characters
    pub strip_spinner_chars: bool,
    /// Strip progress bar block characters
    pub strip_progress_blocks: bool,
    /// Time gap threshold for segment boundaries (seconds)
    pub segment_time_gap: f64,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            strip_ansi: true,
            strip_control_chars: true,
            dedupe_progress_lines: true,
            normalize_whitespace: true,
            max_consecutive_newlines: 2,
            strip_box_drawing: true,
            strip_spinner_chars: true,
            strip_progress_blocks: true,
            segment_time_gap: 2.0,
        }
    }
}
