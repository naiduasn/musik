//! Audio playback using cpal

use crate::{Result, Error, AudioBuffer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Audio player for real-time playback
pub struct AudioPlayer {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| Error::audio("No output device available"))?;

        let config = device
            .default_output_config()
            .map_err(|e| Error::audio(format!("Failed to get output config: {}", e)))?;

        tracing::info!(
            "Audio output: {} channels @ {} Hz",
            config.channels(),
            config.sample_rate().0
        );

        Ok(Self {
            device,
            config: config.into(),
        })
    }

    /// Play an audio buffer (blocks until complete)
    pub fn play(&self, buffer: &AudioBuffer) -> Result<()> {
        tracing::info!("Playing audio: {} frames", buffer.num_frames());

        // Create a shared buffer for the stream
        let data = Arc::new(Mutex::new(buffer.data.clone()));
        let position = Arc::new(Mutex::new(0usize));
        let channels = buffer.channels;

        let data_clone = data.clone();
        let position_clone = position.clone();

        // Build output stream
        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let data = data_clone.lock().unwrap();
                    let mut pos = position_clone.lock().unwrap();

                    for sample in output.iter_mut() {
                        if *pos < data.len() {
                            *sample = data[*pos];
                            *pos += 1;
                        } else {
                            *sample = 0.0;
                        }
                    }
                },
                |err| {
                    tracing::error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| Error::audio(format!("Failed to build output stream: {}", e)))?;

        stream
            .play()
            .map_err(|e| Error::audio(format!("Failed to play stream: {}", e)))?;

        // Wait until playback is done
        loop {
            let pos = *position.lock().unwrap();
            if pos >= buffer.data.len() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Add small delay for buffer drain
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    /// Get number of channels
    pub fn channels(&self) -> u16 {
        self.config.channels
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create audio player")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_creation() {
        // This may fail in CI environments without audio devices
        if let Ok(player) = AudioPlayer::new() {
            assert!(player.sample_rate() > 0);
            assert!(player.channels() > 0);
        }
    }

    #[test]
    fn test_play_silence() {
        if let Ok(player) = AudioPlayer::new() {
            let buffer = AudioBuffer {
                sample_rate: player.sample_rate(),
                channels: player.channels(),
                data: vec![0.0; 4410], // 0.1 seconds of silence
            };

            // This may fail in CI, so we just test it doesn't panic
            let _ = player.play(&buffer);
        }
    }
}
