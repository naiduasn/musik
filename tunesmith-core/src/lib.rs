//! TuneSmith Core - Local music composition and rendering engine
//!
//! This library provides the core functionality for TuneSmith Local:
//! - Composition engine (harmony, melody, rhythm)
//! - Rendering engine (synthesis, mixing)
//! - Audio I/O (recording, playback)
//! - VO processing (gate, ducking, basic effects)
//! - Export (MIDI, WAV stems, GarageBand .band bundles)

pub mod types;
pub mod error;
pub mod composition;
pub mod rendering;
pub mod audio;
pub mod vo;
pub mod export;
pub mod utils;

// Re-export common types
pub use error::{Error, Result};
pub use types::{
    TuneProgram, TuneConfig, GeneratedTune, AudioBuffer, ExportFormat,
    MidiTrack, MidiEvent, MidiEventType, Instrument,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
