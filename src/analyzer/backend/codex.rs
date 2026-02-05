//! Codex backend implementation.
//!
//! Invokes the Codex CLI with `exec --full-auto` for analysis.
//! Note: Codex doesn't support native JSON output, so we extract JSON from text.

use super::{
    extract_json, parse_rate_limit_info, AgentBackend, BackendError, BackendResult, RawMarker,
};
use crate::analyzer::TokenBudget;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Backend for Codex CLI.
///
/// Uses `codex exec --full-auto` for non-interactive analysis.
/// Since Codex doesn't support JSON output mode, responses need
/// JSON extraction from text.
#[derive(Debug, Clone, Default)]
pub struct CodexBackend;

impl CodexBackend {
    /// Create a new Codex backend.
    pub fn new() -> Self {
        Self
    }

    /// Get the CLI command name.
    fn command() -> &'static str {
        "codex"
    }
}

impl AgentBackend for CodexBackend {
    fn name(&self) -> &'static str {
        "Codex"
    }

    fn is_available(&self) -> bool {
        super::command_exists(Self::command())
    }

    fn invoke(&self, prompt: &str, timeout: Duration) -> BackendResult<String> {
        if !self.is_available() {
            return Err(BackendError::NotAvailable(
                "codex CLI not found in PATH".to_string(),
            ));
        }

        let mut child = Command::new(Self::command())
            .args(["exec", "--full-auto"])
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
        // Codex doesn't have native JSON output, so we need to extract it
        let analysis = extract_json(response)?;
        Ok(analysis.markers)
    }

    fn token_budget(&self) -> TokenBudget {
        TokenBudget::codex()
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
    fn codex_backend_name() {
        let backend = CodexBackend::new();
        assert_eq!(backend.name(), "Codex");
    }

    #[test]
    fn codex_backend_token_budget() {
        let backend = CodexBackend::new();
        let budget = backend.token_budget();
        assert_eq!(budget.max_input_tokens, 192_000);
    }

    #[test]
    fn codex_backend_parse_direct_json() {
        let backend = CodexBackend::new();
        let response =
            r#"{"markers": [{"timestamp": 10.0, "label": "Test", "category": "success"}]}"#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 1);
    }

    #[test]
    fn codex_backend_parse_json_in_text() {
        let backend = CodexBackend::new();
        let response = r#"I analyzed the session and here are the markers:

{"markers": [
    {"timestamp": 5.5, "label": "Planning phase started", "category": "planning"},
    {"timestamp": 30.0, "label": "Implementation began", "category": "implementation"}
]}

Let me know if you need more details."#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].category, MarkerCategory::Planning);
        assert_eq!(markers[1].category, MarkerCategory::Implementation);
    }

    #[test]
    fn codex_backend_parse_json_in_code_block() {
        let backend = CodexBackend::new();
        let response = r#"Here's the analysis:

```json
{"markers": [
    {"timestamp": 15.0, "label": "Tests started", "category": "implementation"},
    {"timestamp": 25.0, "label": "All tests passed", "category": "success"}
]}
```

Analysis complete."#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[1].label, "All tests passed");
    }

    #[test]
    fn codex_backend_parse_plain_code_block() {
        let backend = CodexBackend::new();
        let response = r#"
```
{"markers": [{"timestamp": 5.0, "label": "Error found", "category": "failure"}]}
```
"#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].category, MarkerCategory::Failure);
    }

    #[test]
    fn codex_backend_parse_empty_markers() {
        let backend = CodexBackend::new();
        let response = r#"No significant events found: {"markers": []}"#;

        let markers = backend.parse_response(response).unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn codex_backend_parse_no_json() {
        let backend = CodexBackend::new();
        let response = "I couldn't analyze the session properly.";

        let result = backend.parse_response(response);
        assert!(matches!(result, Err(BackendError::JsonExtraction { .. })));
    }

    #[test]
    fn codex_backend_parse_malformed_json() {
        let backend = CodexBackend::new();
        let response = r#"{"markers": [{"timestamp": "not a number"}]}"#;

        let result = backend.parse_response(response);
        assert!(result.is_err());
    }
}
