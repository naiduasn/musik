//! GarageBand .band bundle export

use crate::{Result, Error, GeneratedTune, AudioBuffer};
use std::path::Path;
use plist::Value;

/// Export to GarageBand .band bundle
pub fn export_garageband(
    tune: &GeneratedTune,
    audio: Option<&AudioBuffer>,
    output_path: &Path,
) -> Result<()> {
    tracing::info!("Exporting GarageBand bundle to {}", output_path.display());

    // Create .band directory structure
    std::fs::create_dir_all(output_path)
        .map_err(|e| Error::export(format!("Failed to create bundle directory: {}", e)))?;

    let media_dir = output_path.join("Media");
    std::fs::create_dir_all(&media_dir)
        .map_err(|e| Error::export(format!("Failed to create Media directory: {}", e)))?;

    let output_dir = output_path.join("Output");
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| Error::export(format!("Failed to create Output directory: {}", e)))?;

    // Export MIDI file to Media directory
    let midi_path = media_dir.join("composition.mid");
    super::midi::export_midi(tune, &midi_path)?;

    // Export audio if available
    if let Some(audio) = audio {
        let audio_path = output_dir.join("mix.wav");
        super::wav::export_wav(audio, &audio_path)?;
    }

    // Create ProjectData plist
    let project_data = create_project_data(tune)?;
    let project_data_path = output_path.join("ProjectData");
    plist::to_file_binary(&project_data_path, &project_data)
        .map_err(|e| Error::export(format!("Failed to write ProjectData: {}", e)))?;

    // Create DisplayState.plist
    let display_state = create_display_state(tune)?;
    let display_state_path = output_path.join("DisplayState.plist");
    plist::to_file_xml(&display_state_path, &display_state)
        .map_err(|e| Error::export(format!("Failed to write DisplayState: {}", e)))?;

    tracing::info!("GarageBand bundle exported successfully");
    Ok(())
}

/// Create ProjectData plist structure
fn create_project_data(tune: &GeneratedTune) -> Result<Value> {
    let mut dict = plist::Dictionary::new();

    // Basic project info
    dict.insert(
        "tempo".to_string(),
        Value::Real(tune.program.tempo as f64),
    );

    dict.insert(
        "timeSignature".to_string(),
        Value::Array(vec![
            Value::Integer(tune.program.time_signature.0.into()),
            Value::Integer(tune.program.time_signature.1.into()),
        ]),
    );

    dict.insert(
        "key".to_string(),
        Value::String(tune.program.key.clone()),
    );

    // Track list
    let mut tracks = Vec::new();
    for (idx, track) in tune.tracks.iter().enumerate() {
        let mut track_dict = plist::Dictionary::new();
        track_dict.insert("name".to_string(), Value::String(track.name.clone()));
        track_dict.insert("index".to_string(), Value::Integer((idx as i64).into()));
        track_dict.insert(
            "instrument".to_string(),
            Value::String(format!("{:?}", track.instrument)),
        );
        tracks.push(Value::Dictionary(track_dict));
    }
    dict.insert("tracks".to_string(), Value::Array(tracks));

    Ok(Value::Dictionary(dict))
}

/// Create DisplayState.plist structure
fn create_display_state(tune: &GeneratedTune) -> Result<Value> {
    let mut dict = plist::Dictionary::new();

    // Window state
    dict.insert(
        "windowFrame".to_string(),
        Value::String("0 0 1200 800".to_string()),
    );

    // Track display info
    let mut tracks = Vec::new();
    for (idx, track) in tune.tracks.iter().enumerate() {
        let mut track_dict = plist::Dictionary::new();
        track_dict.insert("name".to_string(), Value::String(track.name.clone()));
        track_dict.insert("index".to_string(), Value::Integer((idx as i64).into()));
        track_dict.insert("volume".to_string(), Value::Real(1.0));
        track_dict.insert("pan".to_string(), Value::Real(0.0));
        track_dict.insert("mute".to_string(), Value::Boolean(false));
        track_dict.insert("solo".to_string(), Value::Boolean(false));
        tracks.push(Value::Dictionary(track_dict));
    }
    dict.insert("tracks".to_string(), Value::Array(tracks));

    Ok(Value::Dictionary(dict))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TuneProgram, composition::CompositionEngine};

    #[test]
    fn test_export_garageband() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_project.band");

        let engine = CompositionEngine::new();
        let program = TuneProgram {
            seed: Some(42),
            duration_bars: 2,
            ..Default::default()
        };
        let tune = engine.generate(&program).unwrap();

        let result = export_garageband(&tune, None, &output_path);
        assert!(result.is_ok());

        // Check structure exists
        assert!(output_path.exists());
        assert!(output_path.join("Media").exists());
        assert!(output_path.join("Output").exists());
        assert!(output_path.join("ProjectData").exists());
        assert!(output_path.join("DisplayState.plist").exists());

        // Cleanup
        std::fs::remove_dir_all(output_path).ok();
    }

    #[test]
    fn test_create_project_data() {
        let engine = CompositionEngine::new();
        let program = TuneProgram {
            seed: Some(42),
            duration_bars: 2,
            ..Default::default()
        };
        let tune = engine.generate(&program).unwrap();

        let result = create_project_data(&tune);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(matches!(data, Value::Dictionary(_)));
    }
}
