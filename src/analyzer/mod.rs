//! Content extraction and analysis pipeline for AI agent sessions.
//!
//! This module provides the infrastructure for extracting meaningful content
//! from asciicast recordings for LLM analysis. The pipeline strips ANSI codes,
//! deduplicates progress output, and creates segments with token estimates.
//!
//! # Design Philosophy
//!
//! The extraction pipeline is designed for efficiency with large files (100MB+):
//! - **Single-pass processing**: Content cleaning uses a state machine to avoid
//!   multiple passes over the data
//! - **In-place mutation**: Uses the existing Transform trait for memory efficiency
//! - **Semantic preservation**: Preserves meaningful characters like checkmarks
//!   while stripping visual-only decorations
//!
//! # Module Structure
//!
//! - [`config`] - Pipeline configuration
//! - [`transforms`] - Individual content cleaning transforms
//! - [`extractor`] - Pipeline orchestration and segment creation
//! - [`types`] - Data structures for analysis content and segments

mod config;
mod extractor;
mod transforms;
mod types;

// Re-export main types
pub use config::ExtractionConfig;
pub use extractor::ContentExtractor;
pub use transforms::{ContentCleaner, DeduplicateProgressLines, FilterEmptyEvents, NormalizeWhitespace};
pub use types::{AnalysisContent, AnalysisSegment, ExtractionStats, TokenEstimator};
