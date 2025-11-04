//! Audio recording using cpal

use crate::{Result, Error, AudioBuffer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Audio recorder for capturing microphone input
pub struct AudioRecorder {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl AudioRecorder {
    /// Create a new audio recorder
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| Error::audio("No input device available"))?;

        let config = device
            .default_input_config()
            .map_err(|e| Error::audio(format!("Failed to get input config: {}", e)))?;

        tracing::info!(
            "Audio input: {} channels @ {} Hz",
            config.channels(),
            config.sample_rate().0
        );

        Ok(Self {
            device,
            config: config.into(),
        })
    }

    /// Record audio for a specified duration
    pub fn record(&self, duration_secs: f32) -> Result<AudioBuffer> {
        tracing::info!("Recording audio for {} seconds", duration_secs);

        let sample_rate = self.config.sample_rate.0;
        let channels = self.config.channels;
        let expected_samples = (duration_secs * sample_rate as f32) as usize * channels as usize;

        // Shared buffer for recorded data
        let recorded_data = Arc::new(Mutex::new(Vec::with_capacity(expected_samples)));
        let recorded_data_clone = recorded_data.clone();

        // Build input stream
        let stream = self
            .device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buffer = recorded_data_clone.lock().unwrap();
                    buffer.extend_from_slice(data);
                },
                |err| {
                    tracing::error!("Audio recording error: {}", err);
                },
                None,
            )
            .map_err(|e| Error::audio(format!("Failed to build input stream: {}", e)))?;

        stream
            .play()
            .map_err(|e| Error::audio(format!("Failed to start recording: {}", e)))?;

        // Record for specified duration
        std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));

        // Stop stream
        drop(stream);

        // Get recorded data
        let data = recorded_data.lock().unwrap().clone();

        tracing::info!("Recorded {} samples", data.len());

        Ok(AudioBuffer {
            sample_rate,
            channels,
            data,
        })
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

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new().expect("Failed to create audio recorder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_recorder_creation() {
        // This may fail in CI environments without audio devices
        if let Ok(recorder) = AudioRecorder::new() {
            assert!(recorder.sample_rate() > 0);
            assert!(recorder.channels() > 0);
        }
    }

    #[test]
    fn test_record_short() {
        if let Ok(recorder) = AudioRecorder::new() {
            // Record 0.1 seconds
            if let Ok(buffer) = recorder.record(0.1) {
                assert_eq!(buffer.sample_rate, recorder.sample_rate());
                assert_eq!(buffer.channels, recorder.channels());
                assert!(!buffer.data.is_empty());
            }
        }
    }
}
