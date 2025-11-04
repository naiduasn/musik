// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tunesmith_core::{
    composition::CompositionEngine,
    rendering::RenderingEngine,
    audio::{AudioPlayer, AudioRecorder},
    export::export_tune,
    vo::{process_voiceover, apply_gate},
    rendering::mixer::apply_ducking,
    TuneProgram, TuneConfig, GeneratedTune, AudioBuffer, ExportFormat, Result,
};
use std::sync::Mutex;
use std::path::PathBuf;

/// Application state
struct AppState {
    composition_engine: Mutex<CompositionEngine>,
    current_tune: Mutex<Option<GeneratedTune>>,
    current_audio: Mutex<Option<AudioBuffer>>,
    config: Mutex<TuneConfig>,
}

/// Generate a tune from a program
#[tauri::command]
fn generate_tune(
    program: TuneProgram,
    state: tauri::State<AppState>,
) -> Result<GeneratedTuneResponse> {
    tracing::info!("Generating tune: {}", program.description);

    let engine = state.composition_engine.lock().unwrap();
    let tune = engine.generate(&program)?;

    let response = GeneratedTuneResponse {
        seed: tune.seed,
        num_tracks: tune.tracks.len(),
        duration_bars: tune.program.duration_bars,
    };

    // Store the tune
    *state.current_tune.lock().unwrap() = Some(tune);

    Ok(response)
}

#[derive(serde::Serialize)]
struct GeneratedTuneResponse {
    seed: u64,
    num_tracks: usize,
    duration_bars: u32,
}

/// Render the current tune to audio
#[tauri::command]
fn render_audio(state: tauri::State<AppState>) -> Result<RenderResponse> {
    tracing::info!("Rendering audio");

    let tune_guard = state.current_tune.lock().unwrap();
    let tune = tune_guard
        .as_ref()
        .ok_or_else(|| tunesmith_core::Error::InvalidInput("No tune generated".into()))?;

    let config = state.config.lock().unwrap().clone();
    let mut engine = RenderingEngine::new(&config)?;
    let audio = engine.render(tune, &config)?;

    let duration_secs = audio.duration_secs();

    // Store the audio
    *state.current_audio.lock().unwrap() = Some(audio);

    Ok(RenderResponse { duration_secs })
}

#[derive(serde::Serialize)]
struct RenderResponse {
    duration_secs: f64,
}

/// Play the current audio
#[tauri::command]
fn play_audio(state: tauri::State<AppState>) -> Result<()> {
    tracing::info!("Playing audio");

    let audio_guard = state.current_audio.lock().unwrap();
    let audio = audio_guard
        .as_ref()
        .ok_or_else(|| tunesmith_core::Error::InvalidInput("No audio rendered".into()))?;

    let player = AudioPlayer::new()?;
    player.play(audio)?;

    Ok(())
}

/// Record voice-over
#[tauri::command]
fn record_voiceover(duration_secs: f32, state: tauri::State<AppState>) -> Result<RecordResponse> {
    tracing::info!("Recording voice-over for {} seconds", duration_secs);

    let recorder = AudioRecorder::new()?;
    let mut vo_audio = recorder.record(duration_secs)?;

    // Process voice-over
    process_voiceover(&mut vo_audio, 0.02, true)?;

    let num_samples = vo_audio.data.len();

    Ok(RecordResponse { num_samples })
}

#[derive(serde::Serialize)]
struct RecordResponse {
    num_samples: usize,
}

/// Export the current tune
#[tauri::command]
fn export_project(
    format: String,
    output_path: String,
    state: tauri::State<AppState>,
) -> Result<()> {
    tracing::info!("Exporting to {} format", format);

    let tune_guard = state.current_tune.lock().unwrap();
    let tune = tune_guard
        .as_ref()
        .ok_or_else(|| tunesmith_core::Error::InvalidInput("No tune generated".into()))?;

    let audio_guard = state.current_audio.lock().unwrap();
    let audio = audio_guard.as_ref();

    let export_format = match format.as_str() {
        "midi" => ExportFormat::Midi,
        "stems" => ExportFormat::Stems,
        "mix" => ExportFormat::Mix,
        "garageband" => ExportFormat::GarageBand,
        _ => return Err(tunesmith_core::Error::InvalidInput(format!("Unknown format: {}", format))),
    };

    let path = PathBuf::from(output_path);
    export_tune(tune, audio, export_format, &path)?;

    Ok(())
}

/// Get current configuration
#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> TuneConfig {
    state.config.lock().unwrap().clone()
}

/// Update configuration
#[tauri::command]
fn update_config(config: TuneConfig, state: tauri::State<AppState>) -> Result<()> {
    *state.config.lock().unwrap() = config;
    Ok(())
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,tunesmith_core=debug,tunesmith_app=debug")
        .init();

    tracing::info!("Starting TuneSmith Local v{}", env!("CARGO_PKG_VERSION"));

    // Create application state
    let state = AppState {
        composition_engine: Mutex::new(CompositionEngine::new()),
        current_tune: Mutex::new(None),
        current_audio: Mutex::new(None),
        config: Mutex::new(TuneConfig::default()),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            generate_tune,
            render_audio,
            play_audio,
            record_voiceover,
            export_project,
            get_config,
            update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
