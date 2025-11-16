//! Rendering engine - converts MIDI to audio

pub mod synth;
pub mod mixer;

use crate::{Result, GeneratedTune, AudioBuffer, TuneConfig};
use synth::Synthesizer;

/// Rendering engine
pub struct RenderingEngine {
    synthesizer: Synthesizer,
}

impl RenderingEngine {
    /// Create a new rendering engine
    pub fn new(config: &TuneConfig) -> Result<Self> {
        let synthesizer = Synthesizer::new(config)?;
        Ok(Self { synthesizer })
    }

    /// Render a tune to audio
    pub fn render(&mut self, tune: &GeneratedTune, config: &TuneConfig) -> Result<AudioBuffer> {
        tracing::info!("Rendering tune with {} tracks", tune.tracks.len());

        // Calculate total duration from MIDI events
        let max_time = tune
            .tracks
            .iter()
            .flat_map(|track| track.events.iter())
            .map(|event| event.time)
            .max()
            .unwrap_or(0);

        // Convert MIDI ticks to seconds (assuming 480 ticks per beat)
        let beats = max_time as f64 / 480.0;
        let seconds = (beats * 60.0) / tune.program.tempo as f64;
        let duration_seconds = seconds + 2.0; // Add 2 seconds for reverb tail

        // Render each track
        let mut buffer = AudioBuffer::new(config.sample_rate, config.channels);
        buffer.data = vec![0.0; (duration_seconds * config.sample_rate as f64) as usize * config.channels as usize];

        for track in &tune.tracks {
            let track_audio = self.synthesizer.render_track(track, &tune.program, config)?;
            mixer::mix_into(&mut buffer, &track_audio)?;
        }

        // Apply master volume
        for sample in &mut buffer.data {
            *sample *= config.master_volume;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::CompositionEngine;
    use crate::TuneProgram;

    #[test]
    fn test_rendering_engine_creation() {
        let config = TuneConfig::default();
        let result = RenderingEngine::new(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_simple_tune() {
        let config = TuneConfig::default();
        let mut engine = RenderingEngine::new(&config).unwrap();

        // Generate a simple tune
        let composer = CompositionEngine::new();
        let program = TuneProgram {
            duration_bars: 2,
            seed: Some(42),
            ..Default::default()
        };
        let tune = composer.generate(&program).unwrap();

        // Render it
        let result = engine.render(&tune, &config);
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert_eq!(audio.sample_rate, config.sample_rate);
        assert_eq!(audio.channels, config.channels);
        assert!(!audio.data.is_empty());
    }
}
