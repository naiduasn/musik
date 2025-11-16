//! Utility functions

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Create a seeded RNG from an optional seed
pub fn create_rng(seed: Option<u64>) -> StdRng {
    match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => {
            let s = rand::thread_rng().gen();
            StdRng::seed_from_u64(s)
        }
    }
}

/// Convert beats to MIDI ticks (480 ticks per quarter note)
pub const TICKS_PER_BEAT: u32 = 480;

pub fn beats_to_ticks(beats: f32) -> u32 {
    (beats * TICKS_PER_BEAT as f32) as u32
}

/// Convert MIDI note number to frequency (Hz)
pub fn note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Clamp a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beats_to_ticks() {
        assert_eq!(beats_to_ticks(1.0), 480);
        assert_eq!(beats_to_ticks(4.0), 1920);
        assert_eq!(beats_to_ticks(0.5), 240);
    }

    #[test]
    fn test_note_to_freq() {
        // A4 = 440 Hz
        assert!((note_to_freq(69) - 440.0).abs() < 0.01);
        // A5 = 880 Hz
        assert!((note_to_freq(81) - 880.0).abs() < 0.01);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-1, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_create_rng() {
        let rng1 = create_rng(Some(42));
        let rng2 = create_rng(Some(42));
        // Same seed should produce same initial state
        assert_eq!(format!("{:?}", rng1), format!("{:?}", rng2));
    }
}
