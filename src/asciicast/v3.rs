// Derived from asciinema (https://github.com/asciinema/asciinema)
// Copyright (c) asciinema authors
// Licensed under GPL-3.0-or-later
// Vendored by AGR project

// Allow clippy warnings and dead code from vendored code to keep it close to upstream
#![allow(clippy::format_in_format_args)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};

use super::util::Quantizer;
use super::{EventData, InternalEvent, InternalHeader, TtyTheme};

#[derive(Deserialize)]
struct V3Header {
    version: u8,
    term: V3Term,
    timestamp: Option<u64>,
    idle_time_limit: Option<f64>,
    command: Option<String>,
    title: Option<String>,
    env: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct V3Term {
    cols: u16,
    rows: u16,
    #[serde(rename = "type")]
    type_: Option<String>,
    version: Option<String>,
    theme: Option<V3Theme>,
}

#[derive(Deserialize, Serialize, Clone)]
struct V3Theme {
    #[serde(deserialize_with = "deserialize_color")]
    fg: RGB8,
    #[serde(deserialize_with = "deserialize_color")]
    bg: RGB8,
    #[serde(deserialize_with = "deserialize_palette")]
    palette: V3Palette,
}

#[derive(Clone)]
struct RGB8(rgb::RGB8);

#[derive(Clone)]
struct V3Palette(Vec<RGB8>);

pub struct V3Encoder {
    prev_time: Duration,
    time_quantizer: Quantizer,
}

impl V3Encoder {
    pub fn new() -> Self {
        Self {
            prev_time: Duration::from_micros(0),
            time_quantizer: Quantizer::new(1_000_000),
        }
    }

    pub fn header(&mut self, header: &InternalHeader) -> Vec<u8> {
        let header: V3Header = header.into();
        let mut data = serde_json::to_string(&header).unwrap().into_bytes();
        data.push(b'\n');

        data
    }

    pub fn event(&mut self, event: &InternalEvent) -> Vec<u8> {
        let mut data = self.serialize_event(event).into_bytes();
        data.push(b'\n');

        data
    }

    fn serialize_event(&mut self, event: &InternalEvent) -> String {
        use EventData::*;

        let (code, data) = match &event.data {
            Output(data) => ('o', self.to_json_string(data)),
            Input(data) => ('i', self.to_json_string(data)),
            Resize(cols, rows) => ('r', self.to_json_string(&format!("{cols}x{rows}"))),
            Marker(data) => ('m', self.to_json_string(data)),
            Exit(data) => ('x', self.to_json_string(&data.to_string())),
            Other(code, data) => (*code, self.to_json_string(data)),
        };

        let dt = event.time - self.prev_time;
        self.prev_time = event.time;
        let dt = Duration::from_nanos(self.time_quantizer.next(dt.as_nanos()) as u64);

        format!(
            "[{}, {}, {}]",
            format_duration(dt),
            self.to_json_string(&code.to_string()),
            data,
        )
    }

    fn to_json_string(&self, s: &str) -> String {
        serde_json::to_string(s).unwrap()
    }
}

impl Default for V3Encoder {
    fn default() -> Self {
        Self::new()
    }
}

fn format_duration(duration: Duration) -> String {
    let time_ms = duration.as_millis();
    let secs = time_ms / 1_000;
    let millis = time_ms % 1_000;

    format!("{}.{}", secs, format!("{:03}", millis))
}

impl serde::Serialize for V3Header {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut len = 2;

        if self.timestamp.is_some() {
            len += 1;
        }

        if self.idle_time_limit.is_some() {
            len += 1;
        }

        if self.command.is_some() {
            len += 1;
        }

        if self.title.is_some() {
            len += 1;
        }

        if self.env.as_ref().is_some_and(|env| !env.is_empty()) {
            len += 1;
        }

        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("version", &3)?;
        map.serialize_entry("term", &self.term)?;

        if let Some(timestamp) = self.timestamp {
            map.serialize_entry("timestamp", &timestamp)?;
        }

        if let Some(limit) = self.idle_time_limit {
            map.serialize_entry("idle_time_limit", &limit)?;
        }

        if let Some(command) = &self.command {
            map.serialize_entry("command", &command)?;
        }

        if let Some(title) = &self.title {
            map.serialize_entry("title", &title)?;
        }

        if let Some(env) = &self.env {
            if !env.is_empty() {
                map.serialize_entry("env", &env)?;
            }
        }
        map.end()
    }
}

impl serde::Serialize for V3Term {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut len = 2;

        if self.type_.is_some() {
            len += 1;
        }

        if self.version.is_some() {
            len += 1;
        }

        if self.theme.is_some() {
            len += 1;
        }

        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("cols", &self.cols)?;
        map.serialize_entry("rows", &self.rows)?;

        if let Some(type_) = &self.type_ {
            map.serialize_entry("type", &type_)?;
        }

        if let Some(version) = &self.version {
            map.serialize_entry("version", &version)?;
        }

        if let Some(theme) = &self.theme {
            map.serialize_entry("theme", &theme)?;
        }

        map.end()
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<RGB8, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    parse_hex_color(value).ok_or(serde::de::Error::custom("invalid hex triplet"))
}

fn parse_hex_color(rgb: &str) -> Option<RGB8> {
    if rgb.len() != 7 {
        return None;
    }

    let r = u8::from_str_radix(&rgb[1..3], 16).ok()?;
    let g = u8::from_str_radix(&rgb[3..5], 16).ok()?;
    let b = u8::from_str_radix(&rgb[5..7], 16).ok()?;

    Some(RGB8(rgb::RGB8::new(r, g, b)))
}

fn deserialize_palette<'de, D>(deserializer: D) -> Result<V3Palette, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &str = Deserialize::deserialize(deserializer)?;
    let mut colors: Vec<RGB8> = value.split(':').filter_map(parse_hex_color).collect();
    let len = colors.len();

    if len == 8 {
        colors.extend_from_within(..);
    } else if len != 16 {
        return Err(serde::de::Error::custom("expected 8 or 16 hex triplets"));
    }

    Ok(V3Palette(colors))
}

impl serde::Serialize for RGB8 {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl fmt::Display for RGB8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "#{:0>2x}{:0>2x}{:0>2x}", self.0.r, self.0.g, self.0.b)
    }
}

impl serde::Serialize for V3Palette {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let palette = self
            .0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(":");

        serializer.serialize_str(&palette)
    }
}

impl From<&InternalHeader> for V3Header {
    fn from(header: &InternalHeader) -> Self {
        V3Header {
            version: 3,
            term: V3Term {
                cols: header.term_cols,
                rows: header.term_rows,
                type_: header.term_type.clone(),
                version: header.term_version.clone(),
                theme: header.term_theme.as_ref().map(|t| t.into()),
            },
            timestamp: header.timestamp,
            idle_time_limit: header.idle_time_limit,
            command: header.command.clone(),
            title: header.title.clone(),
            env: header.env.clone(),
        }
    }
}

impl From<&TtyTheme> for V3Theme {
    fn from(tty_theme: &TtyTheme) -> Self {
        let palette = tty_theme.palette.iter().copied().map(RGB8).collect();

        V3Theme {
            fg: RGB8(tty_theme.fg),
            bg: RGB8(tty_theme.bg),
            palette: V3Palette(palette),
        }
    }
}

impl From<&V3Theme> for TtyTheme {
    fn from(tty_theme: &V3Theme) -> Self {
        let palette = tty_theme.palette.0.iter().map(|c| c.0).collect();

        TtyTheme {
            fg: tty_theme.fg.0,
            bg: tty_theme.bg.0,
            palette,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;
    use std::time::Duration;

    #[test]
    fn format_time() {
        assert_eq!(format_duration(Duration::from_millis(0)), "0.000");
        assert_eq!(format_duration(Duration::from_millis(666)), "0.666");
        assert_eq!(format_duration(Duration::from_millis(1000)), "1.000");
        assert_eq!(format_duration(Duration::from_millis(12345)), "12.345");
    }
}
