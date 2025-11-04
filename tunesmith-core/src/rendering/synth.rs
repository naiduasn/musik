//! Synthesizer using RustySynth

use crate::{Result, MidiTrack, MidiEventType, TuneProgram, AudioBuffer, TuneConfig};

/// A simple synthesizer (we'll use fundsp for MVP as RustySynth requires soundfonts)
pub struct Synthesizer {
    sample_rate: u32,
    channels: u16,
}

impl Synthesizer {
    /// Create a new synthesizer
    pub fn new(config: &TuneConfig) -> Result<Self> {
        Ok(Self {
            sample_rate: config.sample_rate,
            channels: config.channels,
        })
    }

    /// Render a MIDI track to audio
    pub fn render_track(
        &mut self,
        track: &MidiTrack,
        program: &TuneProgram,
        config: &TuneConfig,
    ) -> Result<AudioBuffer> {
        // For MVP, we'll use simple synthesis
        // Calculate duration from max event time
        let max_time = track
            .events
            .iter()
            .map(|e| e.time)
            .max()
            .unwrap_or(0);

        let beats = max_time as f64 / 480.0;
        let seconds = (beats * 60.0) / program.tempo as f64;
        let duration_seconds = seconds + 2.0;

        let num_samples = (duration_seconds * config.sample_rate as f64) as usize;
        let mut buffer = AudioBuffer::new(config.sample_rate, config.channels);
        buffer.data = vec![0.0; num_samples * config.channels as usize];

        // Render each note
        let mut active_notes: std::collections::HashMap<u8, usize> = std::collections::HashMap::new();

        for event in &track.events {
            match event.event_type {
                MidiEventType::NoteOn { note, velocity } => {
                    let start_sample = self.ticks_to_samples(event.time, program.tempo);
                    active_notes.insert(note, start_sample);
                }
                MidiEventType::NoteOff { note } => {
                    if let Some(start_sample) = active_notes.remove(&note) {
                        let end_sample = self.ticks_to_samples(event.time, program.tempo);
                        self.render_note(
                            &mut buffer,
                            note,
                            start_sample,
                            end_sample,
                            track.instrument.is_drums(),
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(buffer)
    }

    /// Convert MIDI ticks to sample position
    fn ticks_to_samples(&self, ticks: u32, tempo: u32) -> usize {
        let beats = ticks as f64 / 480.0;
        let seconds = (beats * 60.0) / tempo as f64;
        (seconds * self.sample_rate as f64) as usize
    }

    /// Render a single note
    fn render_note(
        &self,
        buffer: &mut AudioBuffer,
        note: u8,
        start: usize,
        end: usize,
        is_drums: bool,
    ) {
        let freq = note_to_freq(note);
        let duration = (end - start).min(buffer.num_frames() - start);

        if is_drums {
            // Simple noise burst for drums
            self.render_drum_hit(buffer, start, duration);
        } else {
            // Simple sine wave for melodic instruments
            self.render_tone(buffer, start, duration, freq);
        }
    }

    /// Render a drum hit (noise burst)
    fn render_drum_hit(&self, buffer: &mut AudioBuffer, start: usize, duration: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for i in 0..duration {
            let sample_idx = start + i;
            if sample_idx >= buffer.num_frames() {
                break;
            }

            // Exponential decay envelope
            let t = i as f32 / duration as f32;
            let envelope = (-t * 10.0).exp();

            // Noise
            let noise = rng.gen_range(-1.0..1.0);
            let sample = noise * envelope * 0.3;

            // Write to all channels
            for ch in 0..self.channels {
                let idx = sample_idx * self.channels as usize + ch as usize;
                if idx < buffer.data.len() {
                    buffer.data[idx] += sample;
                }
            }
        }
    }

    /// Render a tone (sine wave)
    fn render_tone(&self, buffer: &mut AudioBuffer, start: usize, duration: usize, freq: f32) {
        for i in 0..duration {
            let sample_idx = start + i;
            if sample_idx >= buffer.num_frames() {
                break;
            }

            let t = i as f32 / self.sample_rate as f32;

            // ADSR envelope (simple)
            let attack = 0.01;
            let decay = 0.1;
            let sustain = 0.7;
            let release = 0.1;

            let total_time = duration as f32 / self.sample_rate as f32;
            let envelope = if t < attack {
                t / attack
            } else if t < attack + decay {
                1.0 - (1.0 - sustain) * ((t - attack) / decay)
            } else if t < total_time - release {
                sustain
            } else {
                sustain * (1.0 - (t - (total_time - release)) / release)
            };

            // Simple sine wave
            let phase = 2.0 * std::f32::consts::PI * freq * t;
            let sample = phase.sin() * envelope * 0.3;

            // Write to all channels
            for ch in 0..self.channels {
                let idx = sample_idx * self.channels as usize + ch as usize;
                if idx < buffer.data.len() {
                    buffer.data[idx] += sample;
                }
            }
        }
    }
}

/// Convert MIDI note to frequency
fn note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Instrument, MidiEvent};

    #[test]
    fn test_synthesizer_creation() {
        let config = TuneConfig::default();
        let result = Synthesizer::new(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_note_to_freq() {
        assert!((note_to_freq(69) - 440.0).abs() < 0.01); // A4
        assert!((note_to_freq(60) - 261.63).abs() < 0.1); // C4
    }

    #[test]
    fn test_ticks_to_samples() {
        let config = TuneConfig::default();
        let synth = Synthesizer::new(&config).unwrap();

        // 480 ticks = 1 beat at 120 BPM = 0.5 seconds = 22050 samples at 44100 Hz
        let samples = synth.ticks_to_samples(480, 120);
        assert_eq!(samples, 22050);
    }
}
