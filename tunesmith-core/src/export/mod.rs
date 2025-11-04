//! Export functionality - MIDI, WAV, GarageBand bundles

pub mod midi;
pub mod wav;
pub mod garageband;

use crate::{Result, GeneratedTune, AudioBuffer, ExportFormat};
use std::path::Path;

/// Export a tune to various formats
pub fn export_tune(
    tune: &GeneratedTune,
    audio: Option<&AudioBuffer>,
    format: ExportFormat,
    output_path: &Path,
) -> Result<()> {
    tracing::info!("Exporting tune to {:?} format", format);

    match format {
        ExportFormat::Midi => {
            midi::export_midi(tune, output_path)?;
        }
        ExportFormat::Stems => {
            if let Some(audio) = audio {
                wav::export_wav(audio, output_path)?;
            } else {
                return Err(crate::Error::export("Audio required for stems export"));
            }
        }
        ExportFormat::Mix => {
            if let Some(audio) = audio {
                wav::export_wav(audio, output_path)?;
            } else {
                return Err(crate::Error::export("Audio required for mix export"));
            }
        }
        ExportFormat::GarageBand => {
            garageband::export_garageband(tune, audio, output_path)?;
        }
    }

    tracing::info!("Export complete: {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TuneProgram, composition::CompositionEngine};

    #[test]
    fn test_export_midi() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_export.mid");

        let engine = CompositionEngine::new();
        let program = TuneProgram {
            seed: Some(42),
            duration_bars: 2,
            ..Default::default()
        };
        let tune = engine.generate(&program).unwrap();

        let result = export_tune(&tune, None, ExportFormat::Midi, &output_path);
        assert!(result.is_ok());

        // Check file was created
        assert!(output_path.exists());

        // Cleanup
        std::fs::remove_file(output_path).ok();
    }
}
