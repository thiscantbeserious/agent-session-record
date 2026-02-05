//! Claude backend implementation.
//!
//! Invokes the Claude CLI with `--print --output-format json` for analysis.

use super::{
    extract_json, parse_rate_limit_info, AgentBackend, BackendError, BackendResult, RawMarker,
};
use crate::analyzer::TokenBudget;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Backend for Claude CLI.
///
/// Uses `claude --print --output-format json` for non-interactive analysis.
/// No permission bypass flags needed - the agent only processes text.
#[derive(Debug, Clone, Default)]
pub struct ClaudeBackend;

impl ClaudeBackend {
    /// Create a new Claude backend.
    pub fn new() -> Self {
        Self
    }

    /// Get the CLI command name.
    fn command() -> &'static str {
        "claude"
    }
}

impl AgentBackend for ClaudeBackend {
    fn name(&self) -> &'static str {
        "Claude"
    }

    fn is_available(&self) -> bool {
        super::command_exists(Self::command())
    }

    fn invoke(&self, prompt: &str, timeout: Duration) -> BackendResult<String> {
        if !self.is_available() {
            return Err(BackendError::NotAvailable(
                "claude CLI not found in PATH".to_string(),
            ));
        }

        let mut child = Command::new(Self::command())
            .args(["--print", "--output-format", "json", "-p"])
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
        TokenBudget::claude()
    }
}

/// Wait for child process with timeout.
///
/// Uses a simple polling approach since std::process doesn't have
/// native timeout support.
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
                // Process finished
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
                // Still running
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

    #[test]
    fn claude_backend_name() {
        let backend = ClaudeBackend::new();
        assert_eq!(backend.name(), "Claude");
    }

    #[test]
    fn claude_backend_token_budget() {
        let backend = ClaudeBackend::new();
        let budget = backend.token_budget();
        assert_eq!(budget.max_input_tokens, 200_000);
    }

    #[test]
    fn claude_backend_parse_valid_response() {
        let backend = ClaudeBackend::new();
        let response = r#"{"markers": [
            {"timestamp": 10.0, "label": "Started planning", "category": "planning"},
            {"timestamp": 45.0, "label": "Build complete", "category": "success"}
        ]}"#;

        let markers = backend.parse_response(response).unwrap();
        assert_eq!(markers.len(), 2);
        assert!((markers[0].timestamp - 10.0).abs() < 0.001);
        assert_eq!(markers[0].label, "Started planning");
    }

    #[test]
    fn claude_backend_parse_empty_markers() {
        let backend = ClaudeBackend::new();
        let response = r#"{"markers": []}"#;

        let markers = backend.parse_response(response).unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn claude_backend_parse_invalid_json() {
        let backend = ClaudeBackend::new();
        let response = "not json at all";

        let result = backend.parse_response(response);
        assert!(result.is_err());
    }

    // Note: Integration tests with actual CLI would go in tests/integration/
}
