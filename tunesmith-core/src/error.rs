//! Error types for TuneSmith Core

use thiserror::Error;

/// Result type alias for TuneSmith operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for TuneSmith Core
#[derive(Error, Debug)]
pub enum Error {
    #[error("Composition error: {0}")]
    Composition(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Audio I/O error: {0}")]
    Audio(String),

    #[error("VO processing error: {0}")]
    VoiceOver(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("MIDI error: {0}")]
    Midi(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Plist error: {0}")]
    Plist(#[from] plist::Error),

    #[error("Audio stream error: {0}")]
    Stream(String),

    #[error("Synthesis error: {0}")]
    Synthesis(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl Error {
    /// Create a composition error
    pub fn composition(msg: impl Into<String>) -> Self {
        Self::Composition(msg.into())
    }

    /// Create a rendering error
    pub fn rendering(msg: impl Into<String>) -> Self {
        Self::Rendering(msg.into())
    }

    /// Create an audio error
    pub fn audio(msg: impl Into<String>) -> Self {
        Self::Audio(msg.into())
    }

    /// Create a voice-over error
    pub fn vo(msg: impl Into<String>) -> Self {
        Self::VoiceOver(msg.into())
    }

    /// Create an export error
    pub fn export(msg: impl Into<String>) -> Self {
        Self::Export(msg.into())
    }
}
