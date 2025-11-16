//! Melody generation

use crate::{Result, TuneProgram, MidiTrack, MidiEvent, MidiEventType, Instrument};
use crate::utils::beats_to_ticks;
use rand::Rng;

/// Generate a melodic track (piano, synth, etc.)
pub fn generate_melodic_track<R: Rng>(
    program: &TuneProgram,
    instrument: Instrument,
    rng: &mut R,
    channel: u8,
) -> Result<MidiTrack> {
    let mut events = Vec::new();

    // Program change
    events.push(MidiEvent {
        time: 0,
        event_type: MidiEventType::ProgramChange {
            program: instrument.midi_program(),
        },
    });

    // Simple pentatonic scale for melody
    let root = parse_key(&program.key);
    let scale = vec![
        root + 48,      // Root (C5)
        root + 50,      // Major second (D5)
        root + 52,      // Major third (E5)
        root + 55,      // Perfect fifth (G5)
        root + 57,      // Major sixth (A5)
    ];

    let beats_per_bar = program.time_signature.0 as f32;
    let total_beats = program.duration_bars as f32 * beats_per_bar;

    // Generate notes with varying rhythms
    let mut current_beat = 0.0;
    while current_beat < total_beats {
        // Random note from scale
        let note = scale[rng.gen_range(0..scale.len())];

        // Random duration (1/4, 1/2, or 1 beat)
        let duration = match rng.gen_range(0..3) {
            0 => 0.25,
            1 => 0.5,
            _ => 1.0,
        };

        // Random velocity based on energy
        let base_velocity = (program.energy * 80.0 + 40.0) as u8;
        let velocity = base_velocity.saturating_add(rng.gen_range(0..20));

        let time = beats_to_ticks(current_beat);

        // Note on
        events.push(MidiEvent {
            time,
            event_type: MidiEventType::NoteOn { note, velocity },
        });

        // Note off
        events.push(MidiEvent {
            time: time + beats_to_ticks(duration * 0.9), // Slight gap
            event_type: MidiEventType::NoteOff { note },
        });

        current_beat += duration;

        // Random pause (10% chance)
        if rng.gen_bool(0.1) {
            current_beat += 0.5;
        }
    }

    Ok(MidiTrack {
        name: format!("{:?}", instrument),
        instrument,
        channel,
        events,
    })
}

/// Parse key string to root MIDI note
fn parse_key(key: &str) -> u8 {
    let key_lower = key.to_lowercase();
    let base = if key_lower.starts_with('c') {
        60
    } else if key_lower.starts_with('d') {
        62
    } else if key_lower.starts_with('e') {
        64
    } else if key_lower.starts_with('f') {
        65
    } else if key_lower.starts_with('g') {
        67
    } else if key_lower.starts_with('a') {
        69
    } else if key_lower.starts_with('b') {
        71
    } else {
        60
    };

    if key_lower.contains('#') || key_lower.contains("sharp") {
        base + 1
    } else if key_lower.contains("flat") {
        base - 1
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_generate_melodic_track() {
        let mut rng = StdRng::seed_from_u64(42);
        let program = TuneProgram {
            duration_bars: 4,
            energy: 0.7,
            ..Default::default()
        };

        let result = generate_melodic_track(&program, Instrument::Piano, &mut rng, 0);
        assert!(result.is_ok());

        let track = result.unwrap();
        assert_eq!(track.instrument, Instrument::Piano);
        assert!(!track.events.is_empty());

        // Count note events
        let note_ons: Vec<_> = track
            .events
            .iter()
            .filter(|e| matches!(e.event_type, MidiEventType::NoteOn { .. }))
            .collect();
        let note_offs: Vec<_> = track
            .events
            .iter()
            .filter(|e| matches!(e.event_type, MidiEventType::NoteOff { .. }))
            .collect();

        // Should have matching note on/off events
        assert_eq!(note_ons.len(), note_offs.len());
    }

    #[test]
    fn test_determinism() {
        let program = TuneProgram {
            duration_bars: 2,
            ..Default::default()
        };

        let mut rng1 = StdRng::seed_from_u64(123);
        let track1 = generate_melodic_track(&program, Instrument::Synth, &mut rng1, 0).unwrap();

        let mut rng2 = StdRng::seed_from_u64(123);
        let track2 = generate_melodic_track(&program, Instrument::Synth, &mut rng2, 0).unwrap();

        // Same seed should produce same output
        assert_eq!(track1.events.len(), track2.events.len());
        for (e1, e2) in track1.events.iter().zip(track2.events.iter()) {
            assert_eq!(e1.time, e2.time);
            assert_eq!(e1.event_type, e2.event_type);
        }
    }
}
