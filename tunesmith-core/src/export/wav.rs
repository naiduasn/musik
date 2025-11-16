//! WAV file export

use crate::{Result, Error, AudioBuffer};
use std::path::Path;

/// Export audio buffer to WAV file
pub fn export_wav(buffer: &AudioBuffer, output_path: &Path) -> Result<()> {
    tracing::info!("Exporting WAV to {}", output_path.display());

    let spec = hound::WavSpec {
        channels: buffer.channels,
        sample_rate: buffer.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(output_path, spec)
        .map_err(|e| Error::export(format!("Failed to create WAV writer: {}", e)))?;

    for &sample in &buffer.data {
        writer
            .write_sample(sample)
            .map_err(|e| Error::export(format!("Failed to write sample: {}", e)))?;
    }

    writer
        .finalize()
        .map_err(|e| Error::export(format!("Failed to finalize WAV: {}", e)))?;

    Ok(())
}

/// Export multiple buffers as separate stem files
pub fn export_stems(
    buffers: &[(String, AudioBuffer)],
    output_dir: &Path,
) -> Result<()> {
    tracing::info!("Exporting {} stems to {}", buffers.len(), output_dir.display());

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)
        .map_err(|e| Error::export(format!("Failed to create output directory: {}", e)))?;

    for (name, buffer) in buffers {
        let filename = format!("{}.wav", name.replace(" ", "_").to_lowercase());
        let output_path = output_dir.join(filename);
        export_wav(buffer, &output_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_wav() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_audio.wav");

        let buffer = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.0; 44100], // 1 second of silence
        };

        let result = export_wav(&buffer, &output_path);
        assert!(result.is_ok());

        // Check file exists
        assert!(output_path.exists());

        // Verify we can read it back
        let reader = hound::WavReader::open(&output_path).unwrap();
        assert_eq!(reader.spec().channels, 2);
        assert_eq!(reader.spec().sample_rate, 44100);

        // Cleanup
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_export_stems() {
        let temp_dir = std::env::temp_dir();
        let output_dir = temp_dir.join("test_stems");

        let buffers = vec![
            (
                "Piano".to_string(),
                AudioBuffer {
                    sample_rate: 44100,
                    channels: 2,
                    data: vec![0.1; 1000],
                },
            ),
            (
                "Bass".to_string(),
                AudioBuffer {
                    sample_rate: 44100,
                    channels: 2,
                    data: vec![0.2; 1000],
                },
            ),
        ];

        let result = export_stems(&buffers, &output_dir);
        assert!(result.is_ok());

        // Check files exist
        assert!(output_dir.join("piano.wav").exists());
        assert!(output_dir.join("bass.wav").exists());

        // Cleanup
        std::fs::remove_dir_all(output_dir).ok();
    }
}
