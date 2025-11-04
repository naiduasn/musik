//! Voice-over processing - gate, ducking, basic effects

use crate::{Result, AudioBuffer};

/// Apply a simple noise gate to remove quiet sections
pub fn apply_gate(buffer: &mut AudioBuffer, threshold: f32) -> Result<()> {
    tracing::info!("Applying noise gate with threshold {}", threshold);

    for i in (0..buffer.data.len()).step_by(buffer.channels as usize) {
        // Calculate RMS for this frame
        let mut rms: f32 = 0.0;
        for ch in 0..buffer.channels {
            let idx = i + ch as usize;
            if idx < buffer.data.len() {
                rms += buffer.data[idx] * buffer.data[idx];
            }
        }
        rms = (rms / buffer.channels as f32).sqrt();

        // If below threshold, mute
        if rms < threshold {
            for ch in 0..buffer.channels {
                let idx = i + ch as usize;
                if idx < buffer.data.len() {
                    buffer.data[idx] = 0.0;
                }
            }
        }
    }

    Ok(())
}

/// Apply simple EQ (boost/cut)
pub fn apply_eq(
    buffer: &mut AudioBuffer,
    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,
) -> Result<()> {
    tracing::info!("Applying EQ: low={}, mid={}, high={}", low_gain, mid_gain, high_gain);

    // Very simple EQ using moving average (not ideal but works for MVP)
    // In production, would use proper biquad filters

    let mut filtered = buffer.data.clone();

    for i in 1..buffer.data.len() - 1 {
        // Low frequencies (average of neighbors)
        let low = (buffer.data[i - 1] + buffer.data[i] + buffer.data[i + 1]) / 3.0;

        // High frequencies (difference from neighbors)
        let high = buffer.data[i] - low;

        // Reconstruct
        filtered[i] = low * low_gain + buffer.data[i] * mid_gain + high * high_gain;
    }

    buffer.data = filtered;

    Ok(())
}

/// Simple compressor/limiter
pub fn apply_compressor(buffer: &mut AudioBuffer, threshold: f32, ratio: f32) -> Result<()> {
    tracing::info!("Applying compressor: threshold={}, ratio={}", threshold, ratio);

    for sample in &mut buffer.data {
        let abs_sample = sample.abs();

        if abs_sample > threshold {
            let over = abs_sample - threshold;
            let compressed = threshold + over / ratio;
            *sample = sample.signum() * compressed;
        }
    }

    Ok(())
}

/// Process voice-over with full chain
pub fn process_voiceover(
    buffer: &mut AudioBuffer,
    gate_threshold: f32,
    enable_eq: bool,
) -> Result<()> {
    tracing::info!("Processing voice-over");

    // Apply gate first
    apply_gate(buffer, gate_threshold)?;

    // Apply EQ if enabled
    if enable_eq {
        // Boost mids slightly for voice clarity
        apply_eq(buffer, 0.8, 1.2, 0.9)?;
    }

    // Apply gentle compression
    apply_compressor(buffer, 0.5, 3.0)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer() -> AudioBuffer {
        AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![
                0.1, 0.1, // Quiet
                0.8, 0.8, // Loud
                0.05, 0.05, // Very quiet
                0.9, 0.9, // Very loud
            ],
        }
    }

    #[test]
    fn test_apply_gate() {
        let mut buffer = create_test_buffer();
        let result = apply_gate(&mut buffer, 0.2);
        assert!(result.is_ok());

        // Very quiet samples should be zeroed
        assert_eq!(buffer.data[4], 0.0);
        assert_eq!(buffer.data[5], 0.0);

        // Loud samples should remain
        assert!(buffer.data[2].abs() > 0.0);
    }

    #[test]
    fn test_apply_compressor() {
        let mut buffer = AudioBuffer {
            sample_rate: 44100,
            channels: 1,
            data: vec![0.3, 0.7, 1.2, -1.5],
        };

        let result = apply_compressor(&mut buffer, 0.5, 2.0);
        assert!(result.is_ok());

        // Values above threshold should be compressed
        assert!(buffer.data[2] < 1.2);
        assert!(buffer.data[3].abs() < 1.5);

        // Values below threshold should be unchanged
        assert!((buffer.data[0] - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_process_voiceover() {
        let mut buffer = create_test_buffer();
        let result = process_voiceover(&mut buffer, 0.2, true);
        assert!(result.is_ok());

        // Just ensure it doesn't crash
        assert!(!buffer.data.is_empty());
    }
}
