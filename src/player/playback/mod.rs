//! Playback logic for the native player.
//!
//! This module handles seeking, marker collection, and playback time management.

mod markers;
mod seeking;

pub use markers::collect_markers;
pub use seeking::{find_event_index_at_time, seek_to_time};
