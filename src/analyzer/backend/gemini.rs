//! Gemini backend implementation.
//!
//! Invokes the Gemini CLI with `--output-format json` for analysis.

use super::{
    extract_json, parse_rate_limit_info, AgentBackend, BackendError, BackendResult, RawMarker,
};
use crate::analyzer::TokenBudget;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Backend for Gemini CLI.
///
/// Uses `gemini --output-format json` for non-interactive analysis.
/// No permission bypass flags needed - the agent only processes text.
#[derive(Debug, Clone, Default)]
pub struct GeminiBackend;

impl GeminiBackend {
    /// Create a new Gemini backend.
    pub fn new() -> Self {
        Self
    }

    /// Get the CLI command name.
    fn command() -> &'static str {
        "gemini"
    }
}

impl AgentBackend for GeminiBackend {
    fn name(&self) -> &'static str {
        "Gemini"
    }

    fn is_available(&self) -> bool {
        super::command_exists(Self::command())
    }

    fn invoke(&self, prompt: &str, timeout: Duration) -> BackendResult<String> {
        if !self.is_available() {
            return Err(BackendError::NotAvailable(
                "gemini CLI not found in PATH".to_string(),
            ));
        }

        let mut child = Command::new(Self::command())
            .args(["--output-format", "json"])
            .arg(prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Wait with timeout
        let timeout_secs = timeout.as_secs();
        let result = wait_with_timeout(&mut child, timeout_secs);

        match result {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                    // Check for rate limiting
                    if let Some(info) = parse_rate_limit_info(&stderr) {
                        return Err(BackendError::RateLimited(info));
                    }

                    Err(BackendError::ExitCode {
                        code: output.status.code().unwrap_or(-1),
                        stderr,
                    })
                }
            }
            Err(_) => {
                // Kill the process if timeout
                let _ = child.kill();
                Err(BackendError::Timeout(timeout))
            }
        }
    }

    fn parse_response(&self, response: &str) -> BackendResult<Vec<RawMarker>> {
        let analysis = extract_json(response)?;
        Ok(analysis.markers)
    }

    fn token_budget(&self) -> TokenBudget {
        TokenBudget::gemini()
    }
}

/// Wait for child process with timeout.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout_secs: u64,
) -> std::io::Result<std::process::Output> {
    use std::thread;
    use std::time::Instant;

    let start = Instant::now();
    let poll_interval = Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed().as_secs() >= timeout_secs {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Process timed out",
                    ));
                }
                thread::sleep(poll_interval);
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::backend::MarkerCategory;

    #[test]
    fn gemini_backend_name() {
        let backend = GeminiBackend::new();
        assert_eq!(backend.name(), "Gemini");
    }

    #[test]
    fn gemini_backend_token_budget() {
        let backend = GeminiBackend::new();
        let budget = backend.token_budget();
        // Gemini has 1M context window
        assert_eq!(budget.max_input_tokens, 1_000_000);
    }

    #[test]
    fn gemini_backend_parse_valid_response() {
        let backend = GeminiBackend::new();
        let response = r#"{"markers": [
            {"timestamp": 10.0, "label": "Analysis started", "category": "planning"},
            {"timestamp": 120.5, "label": "Design decision made", "category": "design"},
            {"timestamp": 300.0, "label": "Feature complete", "category": "success"}
        ]}"#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 3);
        assert_eq!(markers[0].category, MarkerCategory::Planning);
        assert_eq!(markers[1].category, MarkerCategory::Design);
        assert_eq!(markers[2].category, MarkerCategory::Success);
    }

    #[test]
    fn gemini_backend_parse_empty_markers() {
        let backend = GeminiBackend::new();
        let response = r#"{"markers": []}"#;

        let markers = backend.parse_response(response).unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn gemini_backend_parse_with_whitespace() {
        let backend = GeminiBackend::new();
        let response = r#"

        {
            "markers": [
                {"timestamp": 5.0, "label": "Test marker", "category": "implementation"}
            ]
        }

        "#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 1);
    }

    #[test]
    fn gemini_backend_parse_invalid_json() {
        let backend = GeminiBackend::new();
        let response = "This is not valid JSON";

        let result = backend.parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn gemini_backend_large_context_budget() {
        let backend = GeminiBackend::new();
        let budget = backend.token_budget();

        // Gemini should have much more available than Claude/Codex
        let available = budget.available_for_content();
        assert!(available > 800_000); // Should be around 841K
    }
}
