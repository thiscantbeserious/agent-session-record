//! Native asciicast player module
//!
//! Provides functionality for playing back asciicast recordings:
//!
//! - `native`: Full-featured native player (seeking, markers, viewport scrolling)
//!
//! # Architecture
//!
//! The player is organized into submodules:
//! - `state`: PlaybackState struct and shared types (MarkerPosition, InputResult)
//! - `input/`: Keyboard and mouse input handling
//! - `playback/`: Seeking, marker collection, and time management
//! - `render/`: UI rendering (viewport, progress bar, status bar, help, scroll indicators)
//!
//! # Usage
//!
//! ```no_run
//! use agr::player::{play_session, PlaybackResult};
//! use std::path::Path;
//!
//! let result = play_session(Path::new("session.cast")).unwrap();
//! match result {
//!     PlaybackResult::Success(name) => println!("Finished: {}", name),
//!     PlaybackResult::Interrupted => println!("Stopped by user"),
//!     PlaybackResult::Error(e) => eprintln!("Error: {}", e),
//! }
//! ```

pub(crate) mod input;
mod native;
pub(crate) mod playback;
pub mod render;
pub mod state;

pub use native::{play_session, play_session_native, PlaybackResult};
pub use state::{InputResult, MarkerPosition, PlaybackState};
