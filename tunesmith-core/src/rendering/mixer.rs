//! Audio mixing utilities

use crate::{Result, AudioBuffer, Error};

/// Mix one audio buffer into another (additive mixing)
pub fn mix_into(dest: &mut AudioBuffer, src: &AudioBuffer) -> Result<()> {
    if dest.sample_rate != src.sample_rate {
        return Err(Error::rendering("Sample rate mismatch"));
    }

    if dest.channels != src.channels {
        return Err(Error::rendering("Channel count mismatch"));
    }

    // Mix samples (resize dest if needed)
    if src.data.len() > dest.data.len() {
        dest.data.resize(src.data.len(), 0.0);
    }

    for (i, &sample) in src.data.iter().enumerate() {
        if i < dest.data.len() {
            dest.data[i] += sample;
        }
    }

    Ok(())
}

/// Apply ducking: reduce volume of music when VO is present
pub fn apply_ducking(
    music: &mut AudioBuffer,
    vo: &AudioBuffer,
    amount: f32,
    threshold: f32,
) -> Result<()> {
    if music.sample_rate != vo.sample_rate {
        return Err(Error::vo("Sample rate mismatch"));
    }

    if music.channels != vo.channels {
        return Err(Error::vo("Channel count mismatch"));
    }

    let min_len = music.data.len().min(vo.data.len());

    for i in (0..min_len).step_by(music.channels as usize) {
        // Calculate RMS of VO for this frame (simple approach: just check magnitude)
        let mut vo_level = 0.0;
        for ch in 0..music.channels {
            let idx = i + ch as usize;
            if idx < vo.data.len() {
                vo_level += vo.data[idx].abs();
            }
        }
        vo_level /= music.channels as f32;

        // If VO is above threshold, duck the music
        if vo_level > threshold {
            let reduction = 1.0 - amount;
            for ch in 0..music.channels {
                let idx = i + ch as usize;
                if idx < music.data.len() {
                    music.data[idx] *= reduction;
                }
            }
        }
    }

    Ok(())
}

/// Simple limiter to prevent clipping
pub fn apply_limiter(buffer: &mut AudioBuffer, threshold: f32) {
    for sample in &mut buffer.data {
        if *sample > threshold {
            *sample = threshold;
        } else if *sample < -threshold {
            *sample = -threshold;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_into() {
        let mut dest = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.5; 100],
        };

        let src = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.3; 100],
        };

        let result = mix_into(&mut dest, &src);
        assert!(result.is_ok());

        // Check mixing happened
        assert!((dest.data[0] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_mix_into_different_lengths() {
        let mut dest = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.5; 50],
        };

        let src = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.3; 100],
        };

        let result = mix_into(&mut dest, &src);
        assert!(result.is_ok());

        // dest should be resized
        assert_eq!(dest.data.len(), 100);
    }

    #[test]
    fn test_apply_limiter() {
        let mut buffer = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![-2.0, -0.5, 0.5, 2.0],
        };

        apply_limiter(&mut buffer, 1.0);

        assert_eq!(buffer.data[0], -1.0);
        assert_eq!(buffer.data[1], -0.5);
        assert_eq!(buffer.data[2], 0.5);
        assert_eq!(buffer.data[3], 1.0);
    }

    #[test]
    fn test_apply_ducking() {
        let mut music = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![1.0; 100],
        };

        let vo = AudioBuffer {
            sample_rate: 44100,
            channels: 2,
            data: vec![0.5; 100],
        };

        let result = apply_ducking(&mut music, &vo, 0.5, 0.1);
        assert!(result.is_ok());

        // Music should be ducked where VO is present
        assert!(music.data[0] < 1.0);
    }
}
