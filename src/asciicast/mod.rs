// Derived from asciinema (https://github.com/asciinema/asciinema)
// Copyright (c) asciinema authors
// Licensed under GPL-3.0-or-later
// Vendored by AGR project

//! asciicast v3 format parser and writer
//!
//! Reference: https://docs.asciinema.org/manual/asciicast/v3/
//!
//! This module provides types and functions for working with asciicast v3 format files.
//! It is derived from the official asciinema implementation but adapted for AGR's needs.

mod util;
mod v3;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub use v3::V3Encoder;

/// asciicast format version
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Version {
    Three,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Three => write!(f, "3"),
        }
    }
}

/// Terminal theme (colors)
#[derive(Debug, Clone)]
pub struct TtyTheme {
    pub fg: rgb::RGB8,
    pub bg: rgb::RGB8,
    pub palette: Vec<rgb::RGB8>,
}

/// asciicast v3 header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<TermInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<EnvInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_time_limit: Option<f64>,
}

/// Internal header representation (compatible with asciinema crate)
#[derive(Debug, Clone)]
pub struct InternalHeader {
    pub term_cols: u16,
    pub term_rows: u16,
    pub term_type: Option<String>,
    pub term_version: Option<String>,
    pub term_theme: Option<TtyTheme>,
    pub timestamp: Option<u64>,
    pub idle_time_limit: Option<f64>,
    pub command: Option<String>,
    pub title: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

impl Default for InternalHeader {
    fn default() -> Self {
        Self {
            term_cols: 80,
            term_rows: 24,
            term_type: None,
            term_version: None,
            term_theme: None,
            timestamp: None,
            idle_time_limit: None,
            command: None,
            title: None,
            env: None,
        }
    }
}

/// Terminal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub term_type: Option<String>,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvInfo {
    #[serde(rename = "SHELL", skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(rename = "TERM", skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
}

/// Event type codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Output (data written to terminal)
    Output, // "o"
    /// Input (data read from terminal)
    Input, // "i"
    /// Marker (annotation)
    Marker, // "m"
    /// Resize (terminal resize)
    Resize, // "r"
    /// Exit (process exit code)
    Exit, // "x"
}

impl EventType {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "o" => Some(EventType::Output),
            "i" => Some(EventType::Input),
            "m" => Some(EventType::Marker),
            "r" => Some(EventType::Resize),
            "x" => Some(EventType::Exit),
            _ => None,
        }
    }

    pub fn to_code(&self) -> &'static str {
        match self {
            EventType::Output => "o",
            EventType::Input => "i",
            EventType::Marker => "m",
            EventType::Resize => "r",
            EventType::Exit => "x",
        }
    }
}

/// Event data types (derived from asciinema crate)
#[derive(Debug, Clone)]
pub enum EventData {
    Output(String),
    Input(String),
    Resize(u16, u16),
    Marker(String),
    Exit(i32),
    Other(char, String),
}

/// An event in the asciicast file
#[derive(Debug, Clone)]
pub struct Event {
    /// Time offset from previous event (in seconds)
    pub time: f64,
    /// Event type
    pub event_type: EventType,
    /// Event data (output text, marker label, etc.)
    pub data: String,
}

impl Event {
    pub fn new(time: f64, event_type: EventType, data: impl Into<String>) -> Self {
        Self {
            time,
            event_type,
            data: data.into(),
        }
    }

    pub fn output(time: f64, data: impl Into<String>) -> Self {
        Self::new(time, EventType::Output, data)
    }

    pub fn marker(time: f64, label: impl Into<String>) -> Self {
        Self::new(time, EventType::Marker, label)
    }

    pub fn is_output(&self) -> bool {
        self.event_type == EventType::Output
    }

    pub fn is_marker(&self) -> bool {
        self.event_type == EventType::Marker
    }

    /// Parse an event from a JSON line
    pub fn from_json(line: &str) -> Result<Self> {
        let value: serde_json::Value =
            serde_json::from_str(line).context("Failed to parse event JSON")?;

        let arr = value.as_array().context("Event must be a JSON array")?;

        if arr.len() < 3 {
            bail!("Event array must have at least 3 elements");
        }

        let time = arr[0].as_f64().context("Event time must be a number")?;

        let code = arr[1].as_str().context("Event type must be a string")?;

        let event_type =
            EventType::from_code(code).with_context(|| format!("Unknown event type: {}", code))?;

        let data = arr[2]
            .as_str()
            .context("Event data must be a string")?
            .to_string();

        Ok(Event {
            time,
            event_type,
            data,
        })
    }

    /// Convert event to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(&serde_json::json!([
            self.time,
            self.event_type.to_code(),
            self.data
        ]))
        .unwrap()
    }
}

/// Internal event representation with Duration-based time (compatible with asciinema crate)
#[derive(Debug, Clone)]
pub struct InternalEvent {
    pub time: Duration,
    pub data: EventData,
}

impl InternalEvent {
    pub fn output(time: Duration, text: String) -> Self {
        InternalEvent {
            time,
            data: EventData::Output(text),
        }
    }

    pub fn input(time: Duration, text: String) -> Self {
        InternalEvent {
            time,
            data: EventData::Input(text),
        }
    }

    pub fn resize(time: Duration, size: (u16, u16)) -> Self {
        InternalEvent {
            time,
            data: EventData::Resize(size.0, size.1),
        }
    }

    pub fn marker(time: Duration, label: String) -> Self {
        InternalEvent {
            time,
            data: EventData::Marker(label),
        }
    }

    pub fn exit(time: Duration, status: i32) -> Self {
        InternalEvent {
            time,
            data: EventData::Exit(status),
        }
    }
}

/// Complete asciicast file representation
#[derive(Debug, Clone)]
pub struct AsciicastFile {
    pub header: Header,
    pub events: Vec<Event>,
}

impl AsciicastFile {
    /// Create a new asciicast file with the given header
    pub fn new(header: Header) -> Self {
        Self {
            header,
            events: Vec::new(),
        }
    }

    /// Parse an asciicast v3 file from a path
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file =
            fs::File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
        let reader = BufReader::new(file);

        Self::parse_reader(reader)
    }

    /// Parse an asciicast v3 file from a reader
    pub fn parse_reader<R: BufRead>(reader: R) -> Result<Self> {
        let mut lines = reader.lines();

        // First line is the header
        let header_line = lines
            .next()
            .context("File is empty")?
            .context("Failed to read header line")?;

        let header: Header =
            serde_json::from_str(&header_line).context("Failed to parse header")?;

        if header.version != 3 {
            bail!(
                "Only asciicast v3 format is supported (got version {})",
                header.version
            );
        }

        // Remaining lines are events
        let mut events = Vec::new();
        for (line_num, line_result) in lines.enumerate() {
            let line =
                line_result.with_context(|| format!("Failed to read line {}", line_num + 2))?;

            if line.trim().is_empty() {
                continue;
            }

            let event = Event::from_json(&line)
                .with_context(|| format!("Failed to parse event on line {}", line_num + 2))?;
            events.push(event);
        }

        Ok(AsciicastFile { header, events })
    }

    /// Parse from a string
    pub fn parse_str(content: &str) -> Result<Self> {
        let reader = BufReader::new(content.as_bytes());
        Self::parse_reader(reader)
    }

    /// Write the asciicast file to a path
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut file =
            fs::File::create(path).with_context(|| format!("Failed to create file: {:?}", path))?;

        self.write_to(&mut file)
    }

    /// Write the asciicast file to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write header
        let header_json =
            serde_json::to_string(&self.header).context("Failed to serialize header")?;
        writeln!(writer, "{}", header_json)?;

        // Write events
        for event in &self.events {
            writeln!(writer, "{}", event.to_json())?;
        }

        Ok(())
    }

    /// Convert to string
    pub fn to_string(&self) -> Result<String> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get all marker events
    pub fn markers(&self) -> Vec<&Event> {
        self.events.iter().filter(|e| e.is_marker()).collect()
    }

    /// Get all output events
    pub fn outputs(&self) -> Vec<&Event> {
        self.events.iter().filter(|e| e.is_output()).collect()
    }

    /// Calculate cumulative time for each event
    pub fn cumulative_times(&self) -> Vec<f64> {
        let mut times = Vec::with_capacity(self.events.len());
        let mut cumulative = 0.0;
        for event in &self.events {
            cumulative += event.time;
            times.push(cumulative);
        }
        times
    }

    /// Find the insertion index for a marker at the given absolute timestamp
    pub fn find_insertion_index(&self, timestamp: f64) -> usize {
        let cumulative_times = self.cumulative_times();
        for (i, &time) in cumulative_times.iter().enumerate() {
            if time > timestamp {
                return i;
            }
        }
        self.events.len()
    }

    /// Calculate the relative time for insertion at a given index
    pub fn calculate_relative_time(&self, index: usize, absolute_timestamp: f64) -> f64 {
        if index == 0 {
            return absolute_timestamp;
        }

        let cumulative_times = self.cumulative_times();
        let prev_cumulative = cumulative_times.get(index - 1).copied().unwrap_or(0.0);
        absolute_timestamp - prev_cumulative
    }
}

/// Encoder trait for asciicast formats
pub trait Encoder {
    fn header(&mut self, header: &InternalHeader) -> Vec<u8>;
    fn event(&mut self, event: &InternalEvent) -> Vec<u8>;
}

impl Encoder for V3Encoder {
    fn header(&mut self, header: &InternalHeader) -> Vec<u8> {
        self.header(header)
    }

    fn event(&mut self, event: &InternalEvent) -> Vec<u8> {
        self.event(event)
    }
}

/// Create an encoder for the given version
pub fn encoder(version: Version) -> Option<Box<dyn Encoder>> {
    match version {
        Version::Three => Some(Box::new(V3Encoder::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cast() -> &'static str {
        r#"{"version":3,"term":{"cols":80,"rows":24}}
[0.5,"o","$ echo hello\r\n"]
[0.1,"o","hello\r\n"]
[0.2,"o","$ "]"#
    }

    fn cast_with_markers() -> &'static str {
        r#"{"version":3,"term":{"cols":80,"rows":24}}
[0.5,"o","$ make build\r\n"]
[1.0,"m","Build started"]
[2.5,"o","Build complete\r\n"]
[0.1,"m","Build finished"]"#
    }

    #[test]
    fn parse_valid_asciicast() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        assert_eq!(cast.header.version, 3);
        assert_eq!(cast.events.len(), 3);
    }

    #[test]
    fn parse_extracts_output_events() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        let outputs = cast.outputs();
        assert_eq!(outputs.len(), 3);
        assert!(outputs[0].data.contains("echo hello"));
    }

    #[test]
    fn parse_extracts_marker_events() {
        let cast = AsciicastFile::parse_str(cast_with_markers()).unwrap();
        let markers = cast.markers();
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].data, "Build started");
        assert_eq!(markers[1].data, "Build finished");
    }

    #[test]
    fn roundtrip_preserves_data() {
        let original = sample_cast();
        let cast = AsciicastFile::parse_str(original).unwrap();
        let written = cast.to_string().unwrap();
        let reparsed = AsciicastFile::parse_str(&written).unwrap();

        assert_eq!(reparsed.header.version, cast.header.version);
        assert_eq!(reparsed.events.len(), cast.events.len());
        for (orig, reparsed) in cast.events.iter().zip(reparsed.events.iter()) {
            assert_eq!(orig.time, reparsed.time);
            assert_eq!(orig.event_type, reparsed.event_type);
            assert_eq!(orig.data, reparsed.data);
        }
    }

    #[test]
    fn cumulative_times_calculated_correctly() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        let times = cast.cumulative_times();
        assert_eq!(times.len(), 3);
        assert!((times[0] - 0.5).abs() < 0.001);
        assert!((times[1] - 0.6).abs() < 0.001);
        assert!((times[2] - 0.8).abs() < 0.001);
    }

    #[test]
    fn find_insertion_index_at_start() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        assert_eq!(cast.find_insertion_index(0.1), 0);
    }

    #[test]
    fn find_insertion_index_in_middle() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        // Cumulative times: 0.5, 0.6, 0.8
        assert_eq!(cast.find_insertion_index(0.55), 1);
    }

    #[test]
    fn find_insertion_index_at_end() {
        let cast = AsciicastFile::parse_str(sample_cast()).unwrap();
        assert_eq!(cast.find_insertion_index(10.0), 3);
    }

    #[test]
    fn event_type_conversion() {
        assert_eq!(EventType::from_code("o"), Some(EventType::Output));
        assert_eq!(EventType::from_code("i"), Some(EventType::Input));
        assert_eq!(EventType::from_code("m"), Some(EventType::Marker));
        assert_eq!(EventType::from_code("r"), Some(EventType::Resize));
        assert_eq!(EventType::from_code("x"), Some(EventType::Exit));
        assert_eq!(EventType::from_code("z"), None);

        assert_eq!(EventType::Output.to_code(), "o");
        assert_eq!(EventType::Marker.to_code(), "m");
        assert_eq!(EventType::Exit.to_code(), "x");
    }

    #[test]
    fn rejects_non_v3_files() {
        let v2_content = r#"{"version":2,"width":80,"height":24}"#;
        let result = AsciicastFile::parse_str(v2_content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("v3"));
    }
}
