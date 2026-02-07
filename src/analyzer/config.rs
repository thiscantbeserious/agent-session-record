//! Configuration for the content extraction pipeline and analysis service.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the content extraction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Enable similarity-based line collapsing (targets redundant log lines)
    pub collapse_similar_lines: bool,
    /// Similarity threshold (0.0 to 1.0) for collapsing lines
    pub similarity_threshold: f64,
    /// Enable coalescing of rapid, similar events (targets TUI redrawing)
    pub coalesce_events: bool,
    /// Time threshold for event coalescing (seconds)
    pub coalesce_time_threshold: f64,
    /// Enable truncation of large output blocks
    pub truncate_large_blocks: bool,
    /// Max times a specific line can repeat globally across the session
    pub max_line_repeats: usize,
    /// Window size for event hashing (number of events to check for redraws)
    pub event_window_size: usize,
    /// Maximum number of lines in a burst before it's considered a file dump
    pub max_burst_lines: usize,
    /// Maximum size of an output block before truncation (bytes)
    pub max_block_size: usize,
    /// Number of lines to keep at head/tail during truncation
    pub truncation_context_lines: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            strip_ansi: true,
            strip_control_chars: true,
            dedupe_progress_lines: false,
            normalize_whitespace: true,
            max_consecutive_newlines: 2,
            strip_box_drawing: true,
            strip_spinner_chars: true,
            strip_progress_blocks: true,
            segment_time_gap: 2.0,
            collapse_similar_lines: true,
            similarity_threshold: 0.80,
            coalesce_events: true,
            coalesce_time_threshold: 0.2, // 200ms
            max_line_repeats: 10,
            event_window_size: 50,
            max_burst_lines: 500,
            truncate_large_blocks: true,
            max_block_size: 8 * 1024, // 8KB
            truncation_context_lines: 50,
        }
    }
}

/// Analysis configuration for the `analyze` command.
///
/// All fields are optional so users only need to specify what they want
/// to override. CLI flags take priority over config, which overrides defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Preferred agent for analysis ("claude", "codex", "gemini")
    #[serde(default = "default_analysis_agent")]
    pub agent: Option<String>,
    /// Number of parallel workers (None = auto-scale)
    #[serde(default)]
    pub workers: Option<usize>,
    /// Timeout per chunk in seconds
    #[serde(default = "default_analysis_timeout")]
    pub timeout: Option<u64>,
    /// Fast mode (skip JSON schema enforcement)
    #[serde(default = "default_analysis_fast")]
    pub fast: Option<bool>,
    /// Auto-curate markers when count exceeds threshold
    #[serde(default = "default_analysis_curate")]
    pub curate: Option<bool>,
}

pub fn default_analysis_agent() -> Option<String> {
    None
}

pub fn default_analysis_timeout() -> Option<u64> {
    Some(120)
}

pub fn default_analysis_fast() -> Option<bool> {
    Some(false)
}

pub fn default_analysis_curate() -> Option<bool> {
    Some(true)
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            agent: default_analysis_agent(),
            workers: None,
            timeout: default_analysis_timeout(),
            fast: default_analysis_fast(),
            curate: default_analysis_curate(),
        }
    }
}

impl AnalysisConfig {
    /// Validate configuration values.
    ///
    /// Returns `Ok(())` if all values are within acceptable bounds,
    /// or an error describing the first invalid value found.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref agent) = self.agent {
            let valid = ["claude", "codex", "gemini"];
            if !valid.contains(&agent.as_str()) {
                return Err(format!(
                    "Unknown agent '{}'. Valid: {}",
                    agent,
                    valid.join(", ")
                ));
            }
        }
        if let Some(0) = self.timeout {
            return Err("analysis.timeout must be > 0".to_string());
        }
        if let Some(t) = self.timeout {
            if t > 3600 {
                return Err(format!("analysis.timeout {} exceeds maximum (3600s)", t));
            }
        }
        if let Some(0) = self.workers {
            return Err("analysis.workers must be > 0".to_string());
        }
        if let Some(w) = self.workers {
            if w > 32 {
                return Err(format!("analysis.workers {} exceeds maximum (32)", w));
            }
        }
        Ok(())
    }

    /// Validate per-agent configs (called from Config level where agents are accessible).
    pub fn validate_agent_configs(
        &self,
        agent_configs: &HashMap<String, AgentAnalysisConfig>,
    ) -> Result<(), String> {
        for (name, agent_config) in agent_configs {
            if let Some(budget) = agent_config.token_budget {
                if budget < 1000 {
                    return Err(format!(
                        "agents.{}.token_budget {} is below minimum (1000)",
                        name, budget
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Per-agent analysis configuration.
///
/// Allows customizing extra CLI arguments and token budgets for individual agents.
/// Each task type (analyze, curate, rename) can override the global `extra_args`.
///
/// ```toml
/// [agents.codex]
/// extra_args = ["--model", "gpt-5.2-codex"]                # default for all tasks
/// analyze_extra_args = ["--model", "gpt-5.2-codex"]        # override for analysis
/// curate_extra_args = ["--model", "gpt-5.1-codex-mini"]    # override for curation
/// rename_extra_args = ["--model", "gpt-5.1-codex-mini"]    # override for rename
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentAnalysisConfig {
    /// Default extra CLI arguments for all tasks
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Extra CLI arguments for analysis (overrides extra_args)
    #[serde(default)]
    pub analyze_extra_args: Vec<String>,
    /// Extra CLI arguments for curation (overrides extra_args)
    #[serde(default)]
    pub curate_extra_args: Vec<String>,
    /// Extra CLI arguments for rename (overrides extra_args)
    #[serde(default)]
    pub rename_extra_args: Vec<String>,
    /// Override the token budget for this agent
    #[serde(default)]
    pub token_budget: Option<usize>,
}

impl AgentAnalysisConfig {
    /// Get effective extra_args for analysis (analyze-specific or global fallback).
    pub fn effective_analyze_args(&self) -> &[String] {
        if self.analyze_extra_args.is_empty() {
            &self.extra_args
        } else {
            &self.analyze_extra_args
        }
    }

    /// Get effective extra_args for curation (curate-specific or global fallback).
    pub fn effective_curate_args(&self) -> &[String] {
        if self.curate_extra_args.is_empty() {
            &self.extra_args
        } else {
            &self.curate_extra_args
        }
    }

    /// Get effective extra_args for rename (rename-specific or global fallback).
    pub fn effective_rename_args(&self) -> &[String] {
        if self.rename_extra_args.is_empty() {
            &self.extra_args
        } else {
            &self.rename_extra_args
        }
    }
}
