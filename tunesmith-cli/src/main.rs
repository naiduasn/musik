//! TuneSmith CLI - Command-line interface for music composition

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tunesmith_core::{
    composition::CompositionEngine,
    rendering::RenderingEngine,
    audio::AudioRecorder,
    export::export_tune,
    vo::process_voiceover,
    TuneProgram, TuneConfig, ExportFormat,
    mood::{Mood, Intensity, SimpleMusicRequest},
    metadata::{MusicLibrary, MusicQuery, MusicMetadata, MusicFiles},
};

#[derive(Parser)]
#[command(name = "tunesmith")]
#[command(about = "TuneSmith Local - Mood-based music generation for content creators", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate music by mood (simple for content creators)
    Mood {
        /// Mood (happy, sad, suspense, thriller, calm, etc.)
        mood: String,

        /// Duration in seconds
        #[arg(short, long)]
        duration: f32,

        /// Intensity (light, medium, intense)
        #[arg(short, long, default_value = "medium")]
        intensity: String,

        /// Output WAV file
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,

        /// Also export MIDI
        #[arg(long)]
        midi: bool,

        /// Optional seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,

        /// Save to library
        #[arg(long)]
        save: bool,
    },

    /// List available moods and their descriptions
    Moods {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Describe a specific mood
        #[arg(short = 'm', long)]
        describe: Option<String>,
    },

    /// Use content creator presets
    Preset {
        /// Preset name (youtube-intro, podcast-bg, social-reel, etc.)
        preset: String,

        /// Output file
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,

        /// Save to library
        #[arg(long)]
        save: bool,
    },

    /// Manage music library
    Library {
        #[command(subcommand)]
        command: LibraryCommands,
    },

    /// Generate a new tune (advanced - requires music knowledge)
    Generate {
        #[arg(short, long)]
        description: String,
        #[arg(short, long, default_value = "pop")]
        style: String,
        #[arg(short, long, default_value = "120")]
        tempo: u32,
        #[arg(short, long, default_value = "C major")]
        key: String,
        #[arg(short, long, default_value = "0.7")]
        energy: f32,
        #[arg(long, default_value = "8")]
        bars: u32,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(short, long, default_value = "output.mid")]
        output: PathBuf,
    },

    /// Record voice-over
    RecordVo {
        #[arg(short, long, default_value = "5.0")]
        duration: f32,
        #[arg(short, long, default_value = "voiceover.wav")]
        output: PathBuf,
        #[arg(long, default_value = "true")]
        gate: bool,
        #[arg(long, default_value = "0.02")]
        threshold: f32,
    },

    /// Complete workflow (legacy - use 'mood' instead)
    Complete {
        #[arg(short, long)]
        description: String,
        #[arg(short, long, default_value = "pop")]
        style: String,
        #[arg(short, long, default_value = "120")]
        tempo: u32,
        #[arg(short, long, default_value = "mix")]
        format: String,
        #[arg(short, long, default_value = "output")]
        output: PathBuf,
    },

    /// Show examples
    Examples,
}

#[derive(Subcommand)]
enum LibraryCommands {
    /// List tracks in library
    List {
        /// Filter by mood
        #[arg(short, long)]
        mood: Option<String>,

        /// Show most popular
        #[arg(short, long)]
        popular: bool,

        /// Show recently generated
        #[arg(short, long)]
        recent: bool,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Search library
    Search {
        /// Mood to search for
        #[arg(short, long)]
        mood: Option<String>,

        /// Use case (e.g., "youtube", "podcast")
        #[arg(short, long)]
        use_case: Option<String>,

        /// Duration range (e.g., "20-40" for 20-40 seconds)
        #[arg(short, long)]
        duration: Option<String>,
    },

    /// Show library statistics
    Stats,

    /// Delete a track
    Delete {
        /// Track ID
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("{}={},tunesmith_core={}", log_level, log_level, log_level))
        .init();

    if let Err(e) = run_command(cli.command) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Mood { mood, duration, intensity, output, midi, seed, save } => {
            cmd_mood(mood, duration, intensity, output, midi, seed, save)
        }
        Commands::Moods { detailed, describe } => {
            cmd_moods(detailed, describe)
        }
        Commands::Preset { preset, output, save } => {
            cmd_preset(preset, output, save)
        }
        Commands::Library { command } => {
            cmd_library(command)
        }
        Commands::Generate { description, style, tempo, key, energy, bars, seed, output } => {
            cmd_generate(description, style, tempo, key, energy, bars, seed, output)
        }
        Commands::RecordVo { duration, output, gate, threshold } => {
            cmd_record_vo(duration, output, gate, threshold)
        }
        Commands::Complete { description, style, tempo, format, output } => {
            cmd_complete(description, style, tempo, format, output)
        }
        Commands::Examples => {
            cmd_examples();
            Ok(())
        }
    }
}

fn cmd_mood(
    mood_str: String,
    duration: f32,
    intensity_str: String,
    output: PathBuf,
    export_midi: bool,
    seed: Option<u64>,
    save: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Generating {} music ({:.1}s)...", mood_str, duration);

    // Parse mood
    let mood = parse_mood(&mood_str)?;
    let intensity = parse_intensity(&intensity_str)?;

    // Create request
    let request = SimpleMusicRequest {
        mood,
        duration_seconds: duration,
        intensity,
        seed,
        loop_seamlessly: false,
    };

    // Generate
    let program = request.to_tune_program()?;
    let composer = CompositionEngine::new();
    let tune = composer.generate(&program)?;

    println!("   {} {} - Generated {} tracks",
        mood.emoji(), format!("{:?}", mood), tune.tracks.len());

    // Render
    println!("🎹 Rendering audio...");
    let config = TuneConfig::default();
    let mut renderer = RenderingEngine::new(&config)?;
    let audio = renderer.render(&tune, &config)?;

    println!("   ✅ Rendered {:.1}s of audio", audio.duration_secs());

    // Export WAV
    tunesmith_core::export::wav::export_wav(&audio, &output)?;
    println!("💾 Saved: {}", output.display());

    // Export MIDI if requested
    let mut midi_path = None;
    if export_midi {
        let midi_file = output.with_extension("mid");
        tunesmith_core::export::midi::export_midi(&tune, &midi_file)?;
        println!("💾 Saved MIDI: {}", midi_file.display());
        midi_path = Some(midi_file);
    }

    // Save to library if requested
    if save {
        let library_path = get_library_path()?;
        let mut library = MusicLibrary::new(library_path)?;

        let metadata = MusicMetadata::from_program(
            &program,
            mood,
            intensity,
            duration,
            tune.seed,
        );

        let files = MusicFiles {
            midi: midi_path,
            wav: Some(output.clone()),
            garageband: None,
        };

        let mut meta_with_files = metadata;
        meta_with_files.files = files;

        let id = library.add_track(meta_with_files)?;
        println!("📚 Saved to library: {}", id);
    }

    Ok(())
}

fn cmd_moods(detailed: bool, describe: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(mood_str) = describe {
        // Describe specific mood
        let mood = parse_mood(&mood_str)?;
        println!("\n{} {} Mood\n{}", mood.emoji(), format!("{:?}", mood).to_uppercase(), "=".repeat(40));
        println!("\n{}\n", mood.description());
        println!("Common Use Cases:");
        for (i, use_case) in mood.use_cases().iter().enumerate() {
            println!("  {}. {}", i + 1, use_case);
        }
        println!();
    } else {
        // List all moods
        println!("\n🎵 Available Moods for Content Creators\n{}", "=".repeat(50));

        if detailed {
            for mood in Mood::all() {
                println!("\n{} {:?}", mood.emoji(), mood);
                println!("   {}", mood.description());
            }
        } else {
            println!("\nPositive & Energetic:");
            for mood in [Mood::Happy, Mood::Uplifting, Mood::Playful, Mood::Romantic, Mood::Inspirational] {
                println!("  {} {:12} - {}", mood.emoji(), format!("{:?}", mood), mood.description());
            }

            println!("\nCalm & Atmospheric:");
            for mood in [Mood::Peaceful, Mood::Calm, Mood::Dreamy, Mood::Ambient] {
                println!("  {} {:12} - {}", mood.emoji(), format!("{:?}", mood), mood.description());
            }

            println!("\nMysterious & Tense:");
            for mood in [Mood::Mysterious, Mood::Suspense, Mood::Thriller, Mood::Dark] {
                println!("  {} {:12} - {}", mood.emoji(), format!("{:?}", mood), mood.description());
            }

            println!("\nEmotional & Dramatic:");
            for mood in [Mood::Sad, Mood::Melancholic, Mood::Fear, Mood::Intense, Mood::Epic] {
                println!("  {} {:12} - {}", mood.emoji(), format!("{:?}", mood), mood.description());
            }

            println!("\nProfessional:");
            for mood in [Mood::Corporate] {
                println!("  {} {:12} - {}", mood.emoji(), format!("{:?}", mood), mood.description());
            }
        }

        println!("\nUse: tunesmith moods --describe <mood> for more details");
        println!("Example: tunesmith mood suspense --duration 30 -o background.wav\n");
    }

    Ok(())
}

fn cmd_preset(preset: String, output: PathBuf, save: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (mood, duration, intensity) = match preset.as_str() {
        "youtube-intro" => (Mood::Uplifting, 10.0, Intensity::Intense),
        "youtube-outro" => (Mood::Happy, 8.0, Intensity::Medium),
        "podcast-intro" => (Mood::Corporate, 15.0, Intensity::Medium),
        "podcast-bg" | "podcast-background" => (Mood::Calm, 120.0, Intensity::Light),
        "social-reel" | "social-media" => (Mood::Playful, 15.0, Intensity::Intense),
        "meditation" => (Mood::Peaceful, 180.0, Intensity::Light),
        "workout" => (Mood::Intense, 240.0, Intensity::Intense),
        "study" => (Mood::Calm, 300.0, Intensity::Light),
        "vlog-bg" | "vlog-background" => (Mood::Uplifting, 120.0, Intensity::Light),
        "trailer" => (Mood::Epic, 30.0, Intensity::Intense),
        "horror" => (Mood::Fear, 60.0, Intensity::Intense),
        "romantic-scene" => (Mood::Romantic, 90.0, Intensity::Medium),
        "documentary" => (Mood::Mysterious, 180.0, Intensity::Medium),
        "corporate-presentation" => (Mood::Corporate, 120.0, Intensity::Medium),
        _ => {
            return Err(format!("Unknown preset '{}'. Try: youtube-intro, podcast-bg, social-reel, meditation, workout, study, trailer, horror", preset).into());
        }
    };

    println!("🎯 Using preset: {}", preset);
    println!("   Mood: {:?}, Duration: {:.0}s, Intensity: {:?}", mood, duration, intensity);

    cmd_mood(
        format!("{:?}", mood).to_lowercase(),
        duration,
        format!("{:?}", intensity).to_lowercase(),
        output,
        false,
        None,
        save,
    )
}

fn cmd_library(command: LibraryCommands) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = get_library_path()?;
    let mut library = MusicLibrary::new(library_path)?;

    match command {
        LibraryCommands::List { mood, popular, recent, limit } => {
            println!("\n📚 Music Library\n{}", "=".repeat(50));

            let tracks = if popular {
                library.get_popular(limit)
            } else if recent {
                library.get_recent(limit)
            } else if let Some(mood_str) = mood {
                let mood = parse_mood(&mood_str)?;
                let query = MusicQuery {
                    mood: Some(mood),
                    ..Default::default()
                };
                library.search(&query)
            } else {
                library.get_recent(limit)
            };

            if tracks.is_empty() {
                println!("No tracks found.");
            } else {
                for (i, track) in tracks.iter().enumerate() {
                    println!("\n{}. {} {:?} - {:.1}s [{:?}]",
                        i + 1,
                        track.mood.emoji(),
                        track.mood,
                        track.duration_seconds,
                        track.intensity
                    );
                    println!("   ID: {}", track.id);
                    println!("   Style: {}, Tempo: {} BPM, Key: {}", track.style, track.tempo, track.key);
                    if track.usage_count > 0 {
                        println!("   Used {} times", track.usage_count);
                    }
                }
            }
            println!();
        }

        LibraryCommands::Search { mood, use_case, duration } => {
            let query = MusicQuery {
                mood: mood.as_ref().and_then(|m| parse_mood(m).ok()),
                use_case,
                duration_range: duration.as_ref().and_then(|d| parse_duration_range(d.as_str())),
                ..Default::default()
            };

            let results = library.search(&query);
            println!("\n🔍 Search Results: {} tracks\n{}", results.len(), "=".repeat(50));

            for (i, track) in results.iter().enumerate() {
                println!("{}. {} {:?} - {:.1}s", i + 1, track.mood.emoji(), track.mood, track.duration_seconds);
                println!("   ID: {}", track.id);
            }
            println!();
        }

        LibraryCommands::Stats => {
            let stats = library.stats();
            println!("\n📊 Library Statistics\n{}", "=".repeat(50));
            println!("Total Tracks: {}", stats.total_tracks);
            println!("Total Duration: {:.1} minutes", stats.total_duration_seconds / 60.0);
            println!("Total Usage: {}", stats.total_usage);
            println!("Moods Available: {}", stats.moods);
            println!();
        }

        LibraryCommands::Delete { id } => {
            library.delete_track(&id)?;
            println!("✅ Deleted track: {}", id);
        }
    }

    Ok(())
}

// Helper functions for legacy commands
fn cmd_generate(
    description: String,
    style: String,
    tempo: u32,
    key: String,
    energy: f32,
    bars: u32,
    seed: Option<u64>,
    output: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Generating tune...");

    let program = TuneProgram {
        version: 1,
        description,
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

    println!("✅ Generated {} tracks (seed: {})", tune.tracks.len(), tune.seed);

    tunesmith_core::export::midi::export_midi(&tune, &output)?;
    println!("💾 Saved: {}", output.display());

    Ok(())
}

fn cmd_record_vo(
    duration: f32,
    output: PathBuf,
    gate: bool,
    threshold: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎤 Recording voice-over for {} seconds...", duration);
    println!("   (Speak now!)");

    let recorder = AudioRecorder::new()?;
    let mut vo_buffer = recorder.record(duration)?;

    println!("✅ Recording complete!");

    if gate {
        println!("🔧 Processing audio...");
        process_voiceover(&mut vo_buffer, threshold, true)?;
    }

    tunesmith_core::export::wav::export_wav(&vo_buffer, &output)?;
    println!("💾 Saved: {}", output.display());

    Ok(())
}

fn cmd_complete(
    description: String,
    style: String,
    tempo: u32,
    format: String,
    output: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 TuneSmith Complete Workflow");
    println!("{}", "=".repeat(40));

    println!("\n📝 Step 1: Generating composition...");
    let program = TuneProgram {
        version: 1,
        description,
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
    println!("   ✅ Generated {} tracks", tune.tracks.len());

    println!("\n🎹 Step 2: Rendering audio...");
    let config = TuneConfig::default();
    let mut renderer = RenderingEngine::new(&config)?;
    let audio = renderer.render(&tune, &config)?;
    println!("   ✅ Rendered {:.2}s", audio.duration_secs());

    println!("\n💾 Step 3: Exporting...");
    let export_format = match format.as_str() {
        "midi" => ExportFormat::Midi,
        "mix" => ExportFormat::Mix,
        "stems" => ExportFormat::Stems,
        "garageband" => ExportFormat::GarageBand,
        _ => ExportFormat::Mix,
    };

    export_tune(&tune, Some(&audio), export_format, &output)?;
    println!("   ✅ Exported to: {}", output.display());
    println!("\n🎉 Complete!");

    Ok(())
}

fn cmd_examples() {
    println!(r#"
🎵 TuneSmith CLI - Examples for Content Creators
=================================================

SIMPLE MOOD-BASED GENERATION (Recommended):

1. Generate 30s of suspense music:
   tunesmith mood suspense --duration 30 -o background.wav

2. Generate with intensity control:
   tunesmith mood thriller --duration 45 --intensity intense -o intro.wav

3. Happy music with MIDI export:
   tunesmith mood happy --duration 20 --midi -o happy.wav

4. Save to library for reuse:
   tunesmith mood calm --duration 60 --save -o meditation.wav

CONTENT CREATOR PRESETS:

5. YouTube intro (10s, uplifting):
   tunesmith preset youtube-intro -o intro.wav

6. Podcast background (2min, calm):
   tunesmith preset podcast-bg -o podcast.wav

7. Social media reel (15s, playful):
   tunesmith preset social-reel -o reel.wav

8. Workout music (4min, intense):
   tunesmith preset workout -o workout.wav

LIBRARY MANAGEMENT:

9. List all moods:
   tunesmith moods

10. Describe a specific mood:
    tunesmith moods --describe suspense

11. Search library:
    tunesmith library search --mood happy --duration "20-40"

12. Show library stats:
    tunesmith library stats

ADVANCED (Requires Music Knowledge):

13. Generate MIDI with specific parameters:
    tunesmith generate -d "Epic orchestral" -s cinematic -t 140 --bars 16

14. Complete workflow with custom settings:
    tunesmith complete -d "Dark ambient" -s electronic -t 90 -f mix

For more information:
   tunesmith --help
   tunesmith mood --help
   tunesmith moods --list
"#);
}

// Helper functions

fn parse_mood(mood_str: &str) -> Result<Mood, Box<dyn std::error::Error>> {
    let mood_lower = mood_str.to_lowercase();
    Mood::all().into_iter()
        .find(|m| format!("{:?}", m).to_lowercase() == mood_lower)
        .ok_or_else(|| format!("Unknown mood '{}'. Use 'tunesmith moods' to see available moods", mood_str).into())
}

fn parse_intensity(intensity_str: &str) -> Result<Intensity, Box<dyn std::error::Error>> {
    match intensity_str.to_lowercase().as_str() {
        "light" => Ok(Intensity::Light),
        "medium" => Ok(Intensity::Medium),
        "intense" => Ok(Intensity::Intense),
        _ => Err(format!("Unknown intensity '{}'. Use: light, medium, or intense", intensity_str).into()),
    }
}

fn parse_duration_range(range_str: &str) -> Option<(f32, f32)> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() == 2 {
        if let (Ok(min), Ok(max)) = (parts[0].parse(), parts[1].parse()) {
            return Some((min, max));
        }
    }
    None
}

fn get_library_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    Ok(PathBuf::from(home).join(".tunesmith").join("library"))
}
