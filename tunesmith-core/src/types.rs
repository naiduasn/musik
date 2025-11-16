//! Core data structures for TuneSmith

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Version of the TuneProgram schema
pub const TUNE_PROGRAM_VERSION: u32 = 1;

/// A complete tune program describing the musical parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TuneProgram {
    /// Schema version
    pub version: u32,

    /// Brief description or prompt
    pub description: String,

    /// Musical style (e.g., "pop", "jazz", "electronic")
    pub style: String,

    /// Tempo in BPM
    #[serde(default = "default_tempo")]
    pub tempo: u32,

    /// Time signature (numerator, denominator)
    #[serde(default = "default_time_signature")]
    pub time_signature: (u32, u32),

    /// Key signature (C, D, E, F, G, A, B) and mode (major, minor, etc.)
    #[serde(default = "default_key")]
    pub key: String,

    /// Mood/energy level (0.0 = calm, 1.0 = energetic)
    #[serde(default = "default_energy")]
    pub energy: f32,

    /// Duration in bars
    #[serde(default = "default_duration")]
    pub duration_bars: u32,

    /// Optional seed for deterministic generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    /// Instrumentation preferences
    #[serde(default)]
    pub instruments: Vec<Instrument>,
}

fn default_tempo() -> u32 { 120 }
fn default_time_signature() -> (u32, u32) { (4, 4) }
fn default_key() -> String { "C major".to_string() }
fn default_energy() -> f32 { 0.5 }
fn default_duration() -> u32 { 8 }

impl Default for TuneProgram {
    fn default() -> Self {
        Self {
            version: TUNE_PROGRAM_VERSION,
            description: String::new(),
            style: "pop".to_string(),
            tempo: default_tempo(),
            time_signature: default_time_signature(),
            key: default_key(),
            energy: default_energy(),
            duration_bars: default_duration(),
            seed: None,
            instruments: vec![
                Instrument::Piano,
                Instrument::Bass,
                Instrument::Drums,
            ],
        }
    }
}

/// Instrument types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Instrument {
    Piano,
    Bass,
    Drums,
    Guitar,
    Synth,
    Strings,
    Brass,
    Woodwind,
    Pad,
}

impl Instrument {
    /// Get MIDI program number for this instrument
    pub fn midi_program(&self) -> u8 {
        match self {
            Instrument::Piano => 0,        // Acoustic Grand Piano
            Instrument::Bass => 32,        // Acoustic Bass
            Instrument::Drums => 0,        // Drums (channel 10)
            Instrument::Guitar => 24,      // Acoustic Guitar
            Instrument::Synth => 80,       // Square Lead
            Instrument::Strings => 48,     // String Ensemble
            Instrument::Brass => 56,       // Trumpet
            Instrument::Woodwind => 71,    // Clarinet
            Instrument::Pad => 88,         // Pad (New Age)
        }
    }

    /// Check if this is a drum instrument (uses channel 10)
    pub fn is_drums(&self) -> bool {
        matches!(self, Instrument::Drums)
    }
}

/// Configuration for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuneConfig {
    /// Sample rate (Hz)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// Number of audio channels
    #[serde(default = "default_channels")]
    pub channels: u16,

    /// Master volume (0.0 to 1.0)
    #[serde(default = "default_volume")]
    pub master_volume: f32,

    /// Enable voice-over ducking
    #[serde(default)]
    pub enable_ducking: bool,

    /// Ducking amount when VO is present (0.0 to 1.0)
    #[serde(default = "default_ducking_amount")]
    pub ducking_amount: f32,
}

fn default_sample_rate() -> u32 { 44100 }
fn default_channels() -> u16 { 2 }
fn default_volume() -> f32 { 0.8 }
fn default_ducking_amount() -> f32 { 0.3 }

impl Default for TuneConfig {
    fn default() -> Self {
        Self {
            sample_rate: default_sample_rate(),
            channels: default_channels(),
            master_volume: default_volume(),
            enable_ducking: true,
            ducking_amount: default_ducking_amount(),
        }
    }
}

/// A generated tune with MIDI data
#[derive(Debug, Clone)]
pub struct GeneratedTune {
    /// The program used to generate this tune
    pub program: TuneProgram,

    /// MIDI tracks (one per instrument)
    pub tracks: Vec<MidiTrack>,

    /// The seed that was used (either provided or generated)
    pub seed: u64,
}

/// A MIDI track for one instrument
#[derive(Debug, Clone)]
pub struct MidiTrack {
    /// Track name
    pub name: String,

    /// Instrument
    pub instrument: Instrument,

    /// MIDI channel (0-15, channel 10 for drums)
    pub channel: u8,

    /// MIDI events
    pub events: Vec<MidiEvent>,
}

/// A MIDI event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiEvent {
    /// Time in ticks from start
    pub time: u32,

    /// Event type
    pub event_type: MidiEventType,
}

/// MIDI event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiEventType {
    /// Note on (note, velocity)
    NoteOn { note: u8, velocity: u8 },

    /// Note off (note)
    NoteOff { note: u8 },

    /// Program change (program)
    ProgramChange { program: u8 },

    /// Control change (controller, value)
    ControlChange { controller: u8, value: u8 },
}

/// Audio buffer for processing
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Sample rate
    pub sample_rate: u32,

    /// Number of channels
    pub channels: u16,

    /// Interleaved audio samples
    pub data: Vec<f32>,
}

impl AudioBuffer {
    /// Create a new audio buffer
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            data: Vec::new(),
        }
    }

    /// Get number of frames
    pub fn num_frames(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.data.len() / self.channels as usize
        }
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.num_frames() as f64 / self.sample_rate as f64
    }
}

/// Export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// MIDI file
    Midi,

    /// WAV audio stems (one file per instrument)
    Stems,

    /// Mixed WAV audio (single file)
    Mix,

    /// GarageBand .band bundle
    GarageBand,
}
