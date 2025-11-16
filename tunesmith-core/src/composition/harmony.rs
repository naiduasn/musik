//! Harmony generation - chords and bass lines

use crate::{Result, TuneProgram, MidiTrack, MidiEvent, MidiEventType, Instrument};
use crate::utils::{beats_to_ticks, TICKS_PER_BEAT};
use rand::Rng;

/// Generate a bass track following chord progression
pub fn generate_bass_track<R: Rng>(
    program: &TuneProgram,
    rng: &mut R,
    channel: u8,
) -> Result<MidiTrack> {
    let mut events = Vec::new();

    // Program change to bass sound
    events.push(MidiEvent {
        time: 0,
        event_type: MidiEventType::ProgramChange {
            program: Instrument::Bass.midi_program(),
        },
    });

    // Simple chord progression: I-V-vi-IV (common pop progression)
    let root = parse_key(&program.key);
    let chords = vec![
        root,           // I
        root + 7,       // V
        root + 9,       // vi
        root + 5,       // IV
    ];

    let beats_per_bar = program.time_signature.0 as f32;
    let bars_per_chord = 2; // Change chord every 2 bars

    for bar in 0..program.duration_bars {
        let chord_idx = ((bar / bars_per_chord) as usize) % chords.len();
        let bass_note = chords[chord_idx] + 24; // Lower octave for bass

        // Play bass on beats 1 and 3
        for beat_offset in [0.0, 2.0] {
            if beat_offset < beats_per_bar {
                let beat = bar as f32 * beats_per_bar + beat_offset;
                let time = beats_to_ticks(beat);

                // Note on
                events.push(MidiEvent {
                    time,
                    event_type: MidiEventType::NoteOn {
                        note: bass_note,
                        velocity: 80,
                    },
                });

                // Note off after 1 beat
                events.push(MidiEvent {
                    time: time + TICKS_PER_BEAT,
                    event_type: MidiEventType::NoteOff { note: bass_note },
                });
            }
        }
    }

    Ok(MidiTrack {
        name: "Bass".to_string(),
        instrument: Instrument::Bass,
        channel,
        events,
    })
}

/// Parse key string to root MIDI note (C4 = 60)
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
        60 // Default to C
    };

    // Adjust for sharps/flats
    if key_lower.contains('#') || key_lower.contains("sharp") {
        base + 1
    } else if key_lower.contains('b') && !key_lower.starts_with('b') {
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
    fn test_parse_key() {
        assert_eq!(parse_key("C major"), 60);
        assert_eq!(parse_key("D major"), 62);
        assert_eq!(parse_key("G major"), 67);
        assert_eq!(parse_key("C# major"), 61);
    }

    #[test]
    fn test_generate_bass_track() {
        let mut rng = StdRng::seed_from_u64(42);
        let program = TuneProgram {
            duration_bars: 4,
            ..Default::default()
        };

        let result = generate_bass_track(&program, &mut rng, 1);
        assert!(result.is_ok());

        let track = result.unwrap();
        assert_eq!(track.instrument, Instrument::Bass);
        assert!(!track.events.is_empty());

        // Check for program change
        assert!(matches!(
            track.events[0].event_type,
            MidiEventType::ProgramChange { .. }
        ));

        // Check we have note events
        let note_events: Vec<_> = track
            .events
            .iter()
            .filter(|e| matches!(e.event_type, MidiEventType::NoteOn { .. }))
            .collect();
        assert!(!note_events.is_empty());
    }
}
