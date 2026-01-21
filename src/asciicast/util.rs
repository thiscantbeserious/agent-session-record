// Derived from asciinema (https://github.com/asciinema/asciinema)
// Copyright (c) asciinema authors
// Licensed under GPL-3.0-or-later
// Vendored by AGR project

// Allow dead code from vendored code that may not be used in all configurations
#![allow(dead_code)]

use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Deserializer};

pub fn deserialize_time<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    let number = value
        .as_f64()
        .map(|v| v.to_string())
        .ok_or(Error::custom("expected number"))?;

    let parts: Vec<&str> = number.split('.').collect();

    match parts.as_slice() {
        [left, right] => {
            let secs: u64 = left.parse().map_err(Error::custom)?;
            let right = right.trim();

            let micros: u64 = format!("{:0<6}", &right[..(6.min(right.len()))])
                .parse()
                .map_err(Error::custom)?;

            Ok(Duration::from_micros(secs * 1_000_000 + micros))
        }

        [number] => {
            let secs: u64 = number.parse().map_err(Error::custom)?;

            Ok(Duration::from_micros(secs * 1_000_000))
        }

        _ => Err(Error::custom(format!("invalid time format: {value}"))),
    }
}

/// Quantizer using error diffusion based on Bresenham algorithm.
/// It ensures the accumulated error at any point is less than Q/2.
/// (Extracted from asciinema/src/util.rs)
pub struct Quantizer {
    q: i128,
    error: i128,
}

impl Quantizer {
    pub fn new(q: u128) -> Self {
        Quantizer {
            q: q as i128,
            error: 0,
        }
    }

    pub fn next(&mut self, value: u128) -> u128 {
        let error_corrected_value = value as i128 + self.error;
        let steps = (error_corrected_value + self.q / 2) / self.q;
        let quantized_value = steps * self.q;

        self.error = error_corrected_value - quantized_value;
        debug_assert!((self.error).abs() <= self.q / 2);

        quantized_value as u128
    }
}

#[cfg(test)]
mod tests {
    use super::Quantizer;

    #[test]
    fn quantizer() {
        let mut quantizer = Quantizer::new(1_000);

        let input = [
            026692, 540290, 064736, 105951, 171006, 191943, 107942, 128108, 148904, 108973, 211002,
            044701, 489307, 405987, 105028, 194590, 061043, 532296, 319015, 152786, 032578, 005445,
            040542, 000756,
        ];

        let expected = [
            27000, 540000, 65000, 106000, 171000, 192000, 108000, 128000, 149000, 109000, 211000,
            44000, 490000, 406000, 105000, 194000, 61000, 532000, 320000, 152000, 33000, 5000,
            41000, 1000,
        ];

        let mut quantized = Vec::new();
        let mut input_sum = 0;
        let mut quantized_sum = 0;

        for input_value in input {
            let quantized_value = quantizer.next(input_value);
            quantized.push(quantized_value);
            input_sum += input_value;
            quantized_sum += quantized_value;
            let error = (input_sum as i128 - quantized_sum as i128).abs();

            assert!(error <= 500, "error: {error}");
        }

        assert_eq!(quantized, expected);
    }
}
