//! Composition engine - generates musical content

pub mod harmony;
pub mod melody;
pub mod rhythm;

use crate::{Result, TuneProgram, GeneratedTune, MidiTrack, Instrument};
use crate::utils::create_rng;

/// Main composition engine
pub struct CompositionEngine {
    // Future: could hold model references, etc.
}

impl CompositionEngine {
    /// Create a new composition engine
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a tune from a program
    pub fn generate(&self, program: &TuneProgram) -> Result<GeneratedTune> {
        tracing::info!("Generating tune: {}", program.description);

        // Create RNG with seed
        let seed = program.seed.unwrap_or_else(|| rand::random());
        let mut rng = create_rng(Some(seed));

        // Generate tracks for each instrument
        let mut tracks = Vec::new();

        for (idx, instrument) in program.instruments.iter().enumerate() {
            let channel = if instrument.is_drums() { 9 } else { idx as u8 };

            let track = match instrument {
                Instrument::Drums => {
                    rhythm::generate_drum_track(program, &mut rng, channel)?
                }
                Instrument::Bass => {
                    harmony::generate_bass_track(program, &mut rng, channel)?
                }
                _ => {
                    melody::generate_melodic_track(program, *instrument, &mut rng, channel)?
                }
            };

            tracks.push(track);
        }

        Ok(GeneratedTune {
            program: program.clone(),
            tracks,
            seed,
        })
    }
}

impl Default for CompositionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_engine_creation() {
        let engine = CompositionEngine::new();
        // Just test it doesn't panic
        drop(engine);
    }

    #[test]
    fn test_generate_basic_tune() {
        let engine = CompositionEngine::new();
        let program = TuneProgram {
            seed: Some(42), // Deterministic
            ..Default::default()
        };

        let result = engine.generate(&program);
        assert!(result.is_ok());

        let tune = result.unwrap();
        assert_eq!(tune.seed, 42);
        assert!(!tune.tracks.is_empty());
    }
}
