//! MIDI file export

use crate::{Result, Error, GeneratedTune, MidiEventType};
use std::path::Path;

/// Export a tune to MIDI file
pub fn export_midi(tune: &GeneratedTune, output_path: &Path) -> Result<()> {
    tracing::info!("Exporting MIDI to {}", output_path.display());

    // Build MIDI file manually (simple format)
    let mut data = Vec::new();

    // MIDI header
    write_header(&mut data, tune.tracks.len() as u16, 480)?;

    // Tempo track
    write_tempo_track(&mut data, tune.program.tempo)?;

    // Write each track
    for track in &tune.tracks {
        write_track(&mut data, track)?;
    }

    // Write to file
    std::fs::write(output_path, data)
        .map_err(|e| Error::export(format!("Failed to write MIDI file: {}", e)))?;

    Ok(())
}

/// Write MIDI header chunk
fn write_header(data: &mut Vec<u8>, num_tracks: u16, ticks_per_beat: u16) -> Result<()> {
    // MThd chunk
    data.extend_from_slice(b"MThd");
    data.extend_from_slice(&6u32.to_be_bytes()); // Chunk length
    data.extend_from_slice(&1u16.to_be_bytes()); // Format 1 (multiple tracks)
    data.extend_from_slice(&(num_tracks + 1).to_be_bytes()); // +1 for tempo track
    data.extend_from_slice(&ticks_per_beat.to_be_bytes());

    Ok(())
}

/// Write tempo track
fn write_tempo_track(data: &mut Vec<u8>, tempo: u32) -> Result<()> {
    let mut track_data = Vec::new();

    // Time signature (4/4)
    track_data.push(0); // Delta time
    track_data.extend_from_slice(&[0xFF, 0x58, 0x04]); // Time signature meta event
    track_data.extend_from_slice(&[4, 2, 24, 8]); // 4/4

    // Tempo
    track_data.push(0); // Delta time
    track_data.extend_from_slice(&[0xFF, 0x51, 0x03]); // Tempo meta event
    let microseconds_per_beat = 60_000_000 / tempo;
    track_data.extend_from_slice(&[
        ((microseconds_per_beat >> 16) & 0xFF) as u8,
        ((microseconds_per_beat >> 8) & 0xFF) as u8,
        (microseconds_per_beat & 0xFF) as u8,
    ]);

    // End of track
    track_data.push(0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    // Write MTrk chunk
    data.extend_from_slice(b"MTrk");
    data.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    data.extend_from_slice(&track_data);

    Ok(())
}

/// Write a track
fn write_track(data: &mut Vec<u8>, track: &crate::MidiTrack) -> Result<()> {
    let mut track_data = Vec::new();

    // Track name
    track_data.push(0);
    track_data.extend_from_slice(&[0xFF, 0x03]); // Track name meta event
    let name_bytes = track.name.as_bytes();
    write_vlq(&mut track_data, name_bytes.len() as u32);
    track_data.extend_from_slice(name_bytes);

    // Sort events by time
    let mut events = track.events.clone();
    events.sort_by_key(|e| e.time);

    let mut last_time = 0;
    for event in &events {
        let delta = event.time.saturating_sub(last_time);
        write_vlq(&mut track_data, delta);

        match event.event_type {
            MidiEventType::NoteOn { note, velocity } => {
                track_data.push(0x90 | track.channel);
                track_data.push(note);
                track_data.push(velocity);
            }
            MidiEventType::NoteOff { note } => {
                track_data.push(0x80 | track.channel);
                track_data.push(note);
                track_data.push(64);
            }
            MidiEventType::ProgramChange { program } => {
                track_data.push(0xC0 | track.channel);
                track_data.push(program);
            }
            MidiEventType::ControlChange { controller, value } => {
                track_data.push(0xB0 | track.channel);
                track_data.push(controller);
                track_data.push(value);
            }
        }

        last_time = event.time;
    }

    // End of track
    track_data.push(0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    // Write MTrk chunk
    data.extend_from_slice(b"MTrk");
    data.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    data.extend_from_slice(&track_data);

    Ok(())
}

/// Write variable-length quantity
fn write_vlq(data: &mut Vec<u8>, mut value: u32) {
    let mut buffer = [0u8; 4];
    let mut n = 0;

    buffer[n] = (value & 0x7F) as u8;
    n += 1;

    while value >= 0x80 {
        value >>= 7;
        buffer[n] = ((value & 0x7F) | 0x80) as u8;
        n += 1;
    }

    // Write in reverse order
    for i in (0..n).rev() {
        data.push(buffer[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TuneProgram, composition::CompositionEngine};

    #[test]
    fn test_export_midi() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_midi.mid");

        let engine = CompositionEngine::new();
        let program = TuneProgram {
            seed: Some(42),
            duration_bars: 2,
            ..Default::default()
        };
        let tune = engine.generate(&program).unwrap();

        let result = export_midi(&tune, &output_path);
        assert!(result.is_ok());

        // Check file exists and has content
        assert!(output_path.exists());
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);

        // Check MIDI header
        let data = std::fs::read(&output_path).unwrap();
        assert_eq!(&data[0..4], b"MThd");

        // Cleanup
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_write_vlq() {
        let mut data = Vec::new();
        write_vlq(&mut data, 0);
        assert_eq!(data, vec![0]);

        data.clear();
        write_vlq(&mut data, 127);
        assert_eq!(data, vec![127]);

        data.clear();
        write_vlq(&mut data, 128);
        assert_eq!(data, vec![0x81, 0x00]);
    }
}
