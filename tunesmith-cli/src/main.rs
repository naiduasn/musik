//! TuneSmith CLI - Command-line interface for music composition

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tunesmith_core::{
    composition::CompositionEngine,
    rendering::RenderingEngine,
    audio::{AudioPlayer, AudioRecorder},
    export::export_tune,
    vo::process_voiceover,
    rendering::mixer::apply_ducking,
    TuneProgram, TuneConfig, ExportFormat,
};

#[derive(Parser)]
#[command(name = "tunesmith")]
#[command(about = "TuneSmith Local - AI-powered local music composition", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new tune from parameters
    Generate {
        /// Description or prompt for the tune
        #[arg(short, long)]
        description: String,

        /// Musical style (pop, jazz, rock, electronic, classical)
        #[arg(short, long, default_value = "pop")]
        style: String,

        /// Tempo in BPM
        #[arg(short, long, default_value = "120")]
        tempo: u32,

        /// Key signature (e.g., "C major", "D minor")
        #[arg(short, long, default_value = "C major")]
        key: String,

        /// Energy level (0.0 to 1.0)
        #[arg(short, long, default_value = "0.7")]
        energy: f32,

        /// Duration in bars
        #[arg(long, default_value = "8")]
        bars: u32,

        /// Optional seed for deterministic generation
        #[arg(long)]
        seed: Option<u64>,

        /// Output path for MIDI file
        #[arg(short, long, default_value = "output.mid")]
        output: PathBuf,
    },

    /// Render MIDI to audio
    Render {
        /// Input MIDI file (or use previously generated)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output WAV file
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,

        /// Sample rate
        #[arg(long, default_value = "44100")]
        sample_rate: u32,
    },

    /// Record voice-over
    RecordVo {
        /// Duration in seconds
        #[arg(short, long, default_value = "5.0")]
        duration: f32,

        /// Output WAV file
        #[arg(short, long, default_value = "voiceover.wav")]
        output: PathBuf,

        /// Apply noise gate
        #[arg(long, default_value = "true")]
        gate: bool,

        /// Gate threshold
        #[arg(long, default_value = "0.02")]
        threshold: f32,
    },

    /// Complete workflow: generate, render, and export
    Complete {
        /// Description for the tune
        #[arg(short, long)]
        description: String,

        /// Musical style
        #[arg(short, long, default_value = "pop")]
        style: String,

        /// Tempo in BPM
        #[arg(short, long, default_value = "120")]
        tempo: u32,

        /// Export format (midi, mix, stems, garageband)
        #[arg(short, long, default_value = "mix")]
        format: String,

        /// Output path
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },

    /// Show example usage
    Examples,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("{}={},tunesmith_core={}", log_level, log_level, log_level))
        .init();

    // Run command
    if let Err(e) = run_command(cli.command) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Generate {
            description,
            style,
            tempo,
            key,
            energy,
            bars,
            seed,
            output,
        } => {
            println!("🎵 Generating tune...");

            let program = TuneProgram {
                version: 1,
                description: description.clone(),
                style,
                tempo,
                time_signature: (4, 4),
                key,
                energy,
                duration_bars: bars,
                seed,
                instruments: vec![
                    tunesmith_core::Instrument::Piano,
                    tunesmith_core::Instrument::Bass,
                    tunesmith_core::Instrument::Drums,
                ],
            };

            let engine = CompositionEngine::new();
            let tune = engine.generate(&program)?;

            println!("✅ Generated tune with seed: {}", tune.seed);
            println!("   Tracks: {}", tune.tracks.len());
            println!("   Duration: {} bars", tune.program.duration_bars);

            // Export to MIDI
            tunesmith_core::export::midi::export_midi(&tune, &output)?;
            println!("💾 Saved MIDI to: {}", output.display());

            Ok(())
        }

        Commands::Render {
            input,
            output,
            sample_rate,
        } => {
            if input.is_some() {
                println!("⚠️  Note: Rendering from MIDI file not yet implemented in MVP");
                println!("   Use 'complete' command for full workflow");
                return Ok(());
            }

            println!("❌ No input specified. Use 'complete' command for full workflow.");
            Ok(())
        }

        Commands::RecordVo {
            duration,
            output,
            gate,
            threshold,
        } => {
            println!("🎤 Recording voice-over for {} seconds...", duration);
            println!("   (Speak now!)");

            let recorder = AudioRecorder::new()?;
            let mut vo_buffer = recorder.record(duration)?;

            println!("✅ Recording complete!");

            if gate {
                println!("🔧 Processing audio (applying gate)...");
                process_voiceover(&mut vo_buffer, threshold, true)?;
            }

            tunesmith_core::export::wav::export_wav(&vo_buffer, &output)?;
            println!("💾 Saved to: {}", output.display());

            Ok(())
        }

        Commands::Complete {
            description,
            style,
            tempo,
            format,
            output,
        } => {
            println!("🎵 TuneSmith Complete Workflow");
            println!("================================");

            // Step 1: Generate
            println!("\n📝 Step 1: Generating composition...");
            let program = TuneProgram {
                version: 1,
                description: description.clone(),
                style,
                tempo,
                time_signature: (4, 4),
                key: "C major".to_string(),
                energy: 0.7,
                duration_bars: 8,
                seed: None,
                instruments: vec![
                    tunesmith_core::Instrument::Piano,
                    tunesmith_core::Instrument::Bass,
                    tunesmith_core::Instrument::Drums,
                ],
            };

            let composer = CompositionEngine::new();
            let tune = composer.generate(&program)?;

            println!("   ✅ Generated {} tracks (seed: {})", tune.tracks.len(), tune.seed);

            // Step 2: Render
            println!("\n🎹 Step 2: Rendering audio...");
            let config = TuneConfig {
                sample_rate: 44100,
                channels: 2,
                master_volume: 0.8,
                enable_ducking: false,
                ducking_amount: 0.3,
            };

            let mut renderer = RenderingEngine::new(&config)?;
            let audio = renderer.render(&tune, &config)?;

            println!("   ✅ Rendered {:.2} seconds of audio", audio.duration_secs());

            // Step 3: Export
            println!("\n💾 Step 3: Exporting...");
            let export_format = match format.as_str() {
                "midi" => ExportFormat::Midi,
                "mix" => ExportFormat::Mix,
                "stems" => ExportFormat::Stems,
                "garageband" => ExportFormat::GarageBand,
                _ => {
                    println!("⚠️  Unknown format '{}', defaulting to 'mix'", format);
                    ExportFormat::Mix
                }
            };

            export_tune(&tune, Some(&audio), export_format, &output)?;

            println!("   ✅ Exported to: {}", output.display());
            println!("\n🎉 Complete! Your tune is ready.");

            Ok(())
        }

        Commands::Examples => {
            print_examples();
            Ok(())
        }
    }
}

fn print_examples() {
    println!(r#"
🎵 TuneSmith CLI - Examples
============================

1. Generate a MIDI file:
   tunesmith generate -d "Upbeat summer pop song" -s pop -t 128 -o my_song.mid

2. Generate with specific parameters:
   tunesmith generate \
     --description "Dark electronic ambient" \
     --style electronic \
     --tempo 90 \
     --key "D minor" \
     --energy 0.3 \
     --bars 16 \
     --output ambient.mid

3. Generate with deterministic seed:
   tunesmith generate -d "Happy tune" --seed 12345 -o happy.mid

4. Complete workflow (generate + render + export):
   tunesmith complete -d "Chill jazz track" -s jazz -t 110 -f mix -o my_track.wav

5. Export to GarageBand:
   tunesmith complete -d "Rock anthem" -s rock -f garageband -o my_project.band

6. Record voice-over (requires microphone):
   tunesmith record-vo -d 10 -o narration.wav

7. Record with custom gate threshold:
   tunesmith record-vo -d 5 --threshold 0.05 -o voice.wav

Enable verbose logging for debugging:
   tunesmith -v complete -d "Test song" -o output.wav

For more information:
   tunesmith --help
   tunesmith <command> --help
"#);
}
