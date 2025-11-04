//! Rhythm generation - drums and percussion

use crate::{Result, TuneProgram, MidiTrack, MidiEvent, MidiEventType, Instrument};
use crate::utils::beats_to_ticks;
use rand::Rng;

// General MIDI drum notes
const KICK: u8 = 36;
const SNARE: u8 = 38;
const CLOSED_HAT: u8 = 42;
const OPEN_HAT: u8 = 46;

/// Generate a drum track
pub fn generate_drum_track<R: Rng>(
    program: &TuneProgram,
    rng: &mut R,
    channel: u8,
) -> Result<MidiTrack> {
    let mut events = Vec::new();

    let beats_per_bar = program.time_signature.0 as f32;
    let total_beats = program.duration_bars as f32 * beats_per_bar;

    // Generate a basic drum pattern
    let mut current_beat = 0.0;
    let mut beat_in_bar = 0;

    while current_beat < total_beats {
        let time = beats_to_ticks(current_beat);

        // Kick on beats 1 and 3 (in 4/4)
        if beat_in_bar == 0 || beat_in_bar == 2 {
            add_hit(&mut events, time, KICK, 100);
        }

        // Snare on beats 2 and 4
        if beat_in_bar == 1 || beat_in_bar == 3 {
            add_hit(&mut events, time, SNARE, 90);
        }

        // Hi-hat on every beat
        let hat = if rng.gen_bool(0.9) {
            CLOSED_HAT
        } else {
            OPEN_HAT
        };
        let velocity = (70 + rng.gen_range(0..20)) as u8;
        add_hit(&mut events, time, hat, velocity);

        // Also add hi-hat on off-beats (eighth notes)
        if current_beat + 0.5 < total_beats {
            let off_beat_time = beats_to_ticks(current_beat + 0.5);
            add_hit(&mut events, off_beat_time, CLOSED_HAT, 60);
        }

        current_beat += 1.0;
        beat_in_bar = (beat_in_bar + 1) % program.time_signature.0 as usize;
    }

    Ok(MidiTrack {
        name: "Drums".to_string(),
        instrument: Instrument::Drums,
        channel,
        events,
    })
}

/// Add a drum hit (note on + immediate note off for drums)
fn add_hit(events: &mut Vec<MidiEvent>, time: u32, note: u8, velocity: u8) {
    events.push(MidiEvent {
        time,
        event_type: MidiEventType::NoteOn { note, velocity },
    });

    // Drums typically use very short note durations
    events.push(MidiEvent {
        time: time + 10, // 10 ticks
        event_type: MidiEventType::NoteOff { note },
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_generate_drum_track() {
        let mut rng = StdRng::seed_from_u64(42);
        let program = TuneProgram {
            duration_bars: 2,
            ..Default::default()
        };

        let result = generate_drum_track(&program, &mut rng, 9);
        assert!(result.is_ok());

        let track = result.unwrap();
        assert_eq!(track.instrument, Instrument::Drums);
        assert_eq!(track.channel, 9); // Drums on channel 10 (0-indexed as 9)
        assert!(!track.events.is_empty());

        // Count note events
        let note_ons: Vec<_> = track
            .events
            .iter()
            .filter(|e| matches!(e.event_type, MidiEventType::NoteOn { .. }))
            .collect();

        // Should have plenty of drum hits
        assert!(note_ons.len() > 0);
    }

    #[test]
    fn test_drum_timing() {
        let mut rng = StdRng::seed_from_u64(42);
        let program = TuneProgram {
            duration_bars: 1,
            time_signature: (4, 4),
            ..Default::default()
        };

        let track = generate_drum_track(&program, &mut rng, 9).unwrap();

        // Get all kick drum hits
        let kicks: Vec<_> = track
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type,
                    MidiEventType::NoteOn { note: KICK, .. }
                )
            })
            .collect();

        // Should have kicks on beat 1 and 3 (at least)
        assert!(kicks.len() >= 2);
    }
}
