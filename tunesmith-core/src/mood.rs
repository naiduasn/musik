//! Mood-based music generation for content creators
//!
//! This module provides a simple interface for generating background music
//! based on emotional moods rather than musical theory knowledge.
//!
//! Research-backed mappings from academic studies (2024):
//! - Tempo-energy correlation
//! - Major/minor key emotional associations
//! - Instrumentation for different moods
//! - Tension/release patterns

use crate::{Instrument, TuneProgram, Result, Error};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// Primary moods for content creators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    // Positive Moods
    Happy,
    Uplifting,
    Romantic,
    Playful,
    Peaceful,

    // Neutral/Atmospheric
    Calm,
    Dreamy,
    Mysterious,
    Ambient,

    // Negative/Tense Moods
    Sad,
    Melancholic,
    Suspense,
    Thriller,
    Fear,
    Dark,
    Intense,
    Epic,

    // Content-Specific
    Corporate,
    Inspirational,
}

impl Mood {
    /// Get all available moods
    pub fn all() -> Vec<Mood> {
        vec![
            Mood::Happy, Mood::Uplifting, Mood::Romantic, Mood::Playful, Mood::Peaceful,
            Mood::Calm, Mood::Dreamy, Mood::Mysterious, Mood::Ambient,
            Mood::Sad, Mood::Melancholic, Mood::Suspense, Mood::Thriller,
            Mood::Fear, Mood::Dark, Mood::Intense, Mood::Epic,
            Mood::Corporate, Mood::Inspirational,
        ]
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Mood::Happy => "😊",
            Mood::Uplifting => "🚀",
            Mood::Romantic => "💕",
            Mood::Playful => "🎮",
            Mood::Peaceful => "🕊️",
            Mood::Calm => "😌",
            Mood::Dreamy => "💭",
            Mood::Mysterious => "🔮",
            Mood::Ambient => "🌌",
            Mood::Sad => "😢",
            Mood::Melancholic => "🥀",
            Mood::Suspense => "😰",
            Mood::Thriller => "😱",
            Mood::Fear => "😨",
            Mood::Dark => "🌑",
            Mood::Intense => "💥",
            Mood::Epic => "⚔️",
            Mood::Corporate => "💼",
            Mood::Inspirational => "✨",
        }
    }

    /// Get description for content creators
    pub fn description(&self) -> &'static str {
        match self {
            Mood::Happy => "Upbeat, joyful music perfect for celebrations",
            Mood::Uplifting => "Motivating, energetic music that inspires",
            Mood::Romantic => "Soft, emotional music for love scenes",
            Mood::Playful => "Fun, bouncy music for lighthearted content",
            Mood::Peaceful => "Gentle, soothing music for relaxation",
            Mood::Calm => "Tranquil background music, unobtrusive",
            Mood::Dreamy => "Ethereal, floating soundscapes",
            Mood::Mysterious => "Enigmatic, curious music with questions",
            Mood::Ambient => "Atmospheric background, minimal intrusion",
            Mood::Sad => "Melancholic, emotional music for somber moments",
            Mood::Melancholic => "Reflective, bittersweet emotional tone",
            Mood::Suspense => "Tense, mysterious music building anticipation",
            Mood::Thriller => "High-tension, dramatic music for intense scenes",
            Mood::Fear => "Dark, unsettling music creating unease",
            Mood::Dark => "Brooding, ominous atmospheric music",
            Mood::Intense => "Powerful, dramatic music with high energy",
            Mood::Epic => "Cinematic, grand music for dramatic moments",
            Mood::Corporate => "Professional, modern business background music",
            Mood::Inspirational => "Hopeful, emotional music that motivates",
        }
    }

    /// Get common use cases
    pub fn use_cases(&self) -> Vec<&'static str> {
        match self {
            Mood::Happy => vec!["Birthday videos", "Success stories", "Comedy content"],
            Mood::Uplifting => vec!["Motivational videos", "Workout content", "Achievement highlights"],
            Mood::Romantic => vec!["Wedding videos", "Love stories", "Relationship content"],
            Mood::Playful => vec!["Kids content", "Game videos", "Pet videos"],
            Mood::Peaceful => vec!["Meditation", "Nature documentaries", "Spa content"],
            Mood::Calm => vec!["Study music", "Background for vlogs", "Podcast intros"],
            Mood::Dreamy => vec!["Art videos", "Fantasy content", "Creative showcases"],
            Mood::Mysterious => vec!["True crime", "Investigation videos", "Mystery content"],
            Mood::Ambient => vec!["Timelapse videos", "Long-form content", "Ambient backgrounds"],
            Mood::Sad => vec!["Emotional stories", "Memorial videos", "Drama content"],
            Mood::Melancholic => vec!["Reflective videos", "Nostalgic content", "Art films"],
            Mood::Suspense => vec!["Thriller videos", "Anticipation build-ups", "Mystery reveals"],
            Mood::Thriller => vec!["Action scenes", "Chase sequences", "Intense moments"],
            Mood::Fear => vec!["Horror content", "Scary videos", "Halloween content"],
            Mood::Dark => vec!["Gothic content", "Dark themes", "Noir aesthetics"],
            Mood::Intense => vec!["Sports highlights", "Action sequences", "Dramatic reveals"],
            Mood::Epic => vec!["Cinematic trailers", "Epic moments", "Grand reveals"],
            Mood::Corporate => vec!["Business presentations", "Product demos", "Professional content"],
            Mood::Inspirational => vec!["Success stories", "Transformation videos", "Life coaching"],
        }
    }
}

/// Intensity level for mood expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intensity {
    /// Subtle background music (0.2-0.4 energy)
    Light,
    /// Noticeable but not overwhelming (0.4-0.6 energy)
    Medium,
    /// Dramatic, foreground music (0.6-0.9 energy)
    Intense,
}

impl Intensity {
    /// Get energy range for this intensity
    pub fn energy_range(&self) -> (f32, f32) {
        match self {
            Intensity::Light => (0.2, 0.4),
            Intensity::Medium => (0.4, 0.6),
            Intensity::Intense => (0.6, 0.9),
        }
    }
}

/// Rhythm pattern characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmPattern {
    /// Sparse, minimal rhythm (suspense, ambient)
    Sparse,
    /// Steady, driving beat (pop, rock)
    Steady,
    /// Irregular, unpredictable (tension, thriller)
    Irregular,
}

/// Mood profile with research-backed musical parameters
#[derive(Debug, Clone)]
pub struct MoodProfile {
    pub mood: Mood,
    pub tempo_range: (u32, u32),
    pub preferred_keys: Vec<&'static str>,
    pub base_energy: (f32, f32),
    pub instruments: Vec<Instrument>,
    pub style: &'static str,
    pub rhythm_pattern: RhythmPattern,
    pub use_dissonance: bool,
    pub dynamics_variation: f32,  // 0.0-1.0, how much volume varies
}

impl MoodProfile {
    /// Get profile for a specific mood
    pub fn for_mood(mood: Mood) -> Self {
        match mood {
            // POSITIVE MOODS
            Mood::Happy => MoodProfile {
                mood,
                tempo_range: (120, 140),
                preferred_keys: vec!["C major", "G major", "D major", "F major"],
                base_energy: (0.7, 0.9),
                instruments: vec![Instrument::Piano, Instrument::Guitar, Instrument::Drums, Instrument::Bass],
                style: "pop",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.3,
            },

            Mood::Uplifting => MoodProfile {
                mood,
                tempo_range: (125, 145),
                preferred_keys: vec!["C major", "D major", "E major"],
                base_energy: (0.75, 0.95),
                instruments: vec![Instrument::Synth, Instrument::Piano, Instrument::Strings, Instrument::Drums],
                style: "electronic",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.4,
            },

            Mood::Romantic => MoodProfile {
                mood,
                tempo_range: (70, 90),
                preferred_keys: vec!["C major", "F major", "Bb major"],
                base_energy: (0.4, 0.6),
                instruments: vec![Instrument::Piano, Instrument::Strings, Instrument::Pad],
                style: "classical",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.5,
            },

            Mood::Playful => MoodProfile {
                mood,
                tempo_range: (130, 150),
                preferred_keys: vec!["C major", "G major", "D major"],
                base_energy: (0.6, 0.8),
                instruments: vec![Instrument::Piano, Instrument::Synth, Instrument::Drums],
                style: "pop",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.4,
            },

            Mood::Peaceful => MoodProfile {
                mood,
                tempo_range: (60, 75),
                preferred_keys: vec!["C major", "F major", "G major"],
                base_energy: (0.2, 0.4),
                instruments: vec![Instrument::Piano, Instrument::Pad, Instrument::Strings],
                style: "ambient",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.2,
            },

            // NEUTRAL/ATMOSPHERIC MOODS
            Mood::Calm => MoodProfile {
                mood,
                tempo_range: (65, 80),
                preferred_keys: vec!["C major", "F major", "Eb major"],
                base_energy: (0.25, 0.45),
                instruments: vec![Instrument::Piano, Instrument::Pad, Instrument::Strings],
                style: "ambient",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.15,
            },

            Mood::Dreamy => MoodProfile {
                mood,
                tempo_range: (70, 90),
                preferred_keys: vec!["C major", "D major", "A major"],
                base_energy: (0.3, 0.5),
                instruments: vec![Instrument::Pad, Instrument::Synth, Instrument::Piano],
                style: "ambient",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.3,
            },

            Mood::Mysterious => MoodProfile {
                mood,
                tempo_range: (75, 95),
                preferred_keys: vec!["A minor", "E minor", "D minor"],
                base_energy: (0.35, 0.55),
                instruments: vec![Instrument::Strings, Instrument::Pad, Instrument::Piano],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: true,
                dynamics_variation: 0.4,
            },

            Mood::Ambient => MoodProfile {
                mood,
                tempo_range: (60, 80),
                preferred_keys: vec!["C major", "D minor", "F major"],
                base_energy: (0.2, 0.35),
                instruments: vec![Instrument::Pad, Instrument::Strings],
                style: "ambient",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.1,
            },

            // NEGATIVE/TENSE MOODS
            Mood::Sad => MoodProfile {
                mood,
                tempo_range: (60, 80),
                preferred_keys: vec!["A minor", "E minor", "D minor", "C minor"],
                base_energy: (0.25, 0.45),
                instruments: vec![Instrument::Piano, Instrument::Strings],
                style: "classical",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.4,
            },

            Mood::Melancholic => MoodProfile {
                mood,
                tempo_range: (65, 85),
                preferred_keys: vec!["A minor", "D minor", "E minor"],
                base_energy: (0.3, 0.5),
                instruments: vec![Instrument::Piano, Instrument::Strings, Instrument::Pad],
                style: "classical",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: false,
                dynamics_variation: 0.5,
            },

            Mood::Suspense => MoodProfile {
                mood,
                tempo_range: (80, 100),
                preferred_keys: vec!["D minor", "E minor", "A minor"],
                base_energy: (0.35, 0.55),
                instruments: vec![Instrument::Strings, Instrument::Pad, Instrument::Drums],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Irregular,
                use_dissonance: true,
                dynamics_variation: 0.6,
            },

            Mood::Thriller => MoodProfile {
                mood,
                tempo_range: (110, 130),
                preferred_keys: vec!["D minor", "C minor", "E minor"],
                base_energy: (0.6, 0.8),
                instruments: vec![Instrument::Synth, Instrument::Strings, Instrument::Drums],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Irregular,
                use_dissonance: true,
                dynamics_variation: 0.7,
            },

            Mood::Fear => MoodProfile {
                mood,
                tempo_range: (90, 120),
                preferred_keys: vec!["C minor", "D minor", "F minor"],
                base_energy: (0.5, 0.7),
                instruments: vec![Instrument::Strings, Instrument::Synth, Instrument::Drums],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Irregular,
                use_dissonance: true,
                dynamics_variation: 0.8,
            },

            Mood::Dark => MoodProfile {
                mood,
                tempo_range: (75, 95),
                preferred_keys: vec!["C minor", "D minor", "E minor"],
                base_energy: (0.4, 0.6),
                instruments: vec![Instrument::Synth, Instrument::Strings, Instrument::Pad],
                style: "electronic",
                rhythm_pattern: RhythmPattern::Sparse,
                use_dissonance: true,
                dynamics_variation: 0.5,
            },

            Mood::Intense => MoodProfile {
                mood,
                tempo_range: (130, 160),
                preferred_keys: vec!["D minor", "E minor", "C minor"],
                base_energy: (0.75, 0.95),
                instruments: vec![Instrument::Synth, Instrument::Drums, Instrument::Bass, Instrument::Strings],
                style: "electronic",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: true,
                dynamics_variation: 0.6,
            },

            Mood::Epic => MoodProfile {
                mood,
                tempo_range: (130, 150),
                preferred_keys: vec!["C major", "D major", "E minor"],
                base_energy: (0.8, 1.0),
                instruments: vec![Instrument::Brass, Instrument::Strings, Instrument::Drums, Instrument::Synth],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.7,
            },

            // CONTENT-SPECIFIC MOODS
            Mood::Corporate => MoodProfile {
                mood,
                tempo_range: (100, 120),
                preferred_keys: vec!["C major", "G major", "F major"],
                base_energy: (0.5, 0.7),
                instruments: vec![Instrument::Piano, Instrument::Synth, Instrument::Drums],
                style: "pop",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.3,
            },

            Mood::Inspirational => MoodProfile {
                mood,
                tempo_range: (90, 110),
                preferred_keys: vec!["C major", "D major", "G major"],
                base_energy: (0.6, 0.8),
                instruments: vec![Instrument::Piano, Instrument::Strings, Instrument::Synth, Instrument::Drums],
                style: "cinematic",
                rhythm_pattern: RhythmPattern::Steady,
                use_dissonance: false,
                dynamics_variation: 0.5,
            },
        }
    }

    /// Apply intensity modification to base energy
    pub fn apply_intensity(&self, intensity: Intensity) -> (f32, f32) {
        let intensity_range = intensity.energy_range();
        let (base_min, base_max) = self.base_energy;

        // Blend base energy with intensity preference
        let min = (base_min + intensity_range.0) / 2.0;
        let max = (base_max + intensity_range.1) / 2.0;

        (min.max(0.1).min(0.95), max.max(0.2).min(1.0))
    }

    /// Get random tempo within range
    pub fn random_tempo<R: Rng>(&self, rng: &mut R) -> u32 {
        rng.gen_range(self.tempo_range.0..=self.tempo_range.1)
    }

    /// Get random key from preferred keys
    pub fn random_key<R: Rng>(&self, rng: &mut R) -> String {
        self.preferred_keys[rng.gen_range(0..self.preferred_keys.len())].to_string()
    }

    /// Get random energy within range
    pub fn random_energy<R: Rng>(&self, rng: &mut R, intensity: Intensity) -> f32 {
        let (min, max) = self.apply_intensity(intensity);
        rng.gen_range(min..=max)
    }
}

/// Simple request for content creators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMusicRequest {
    /// Emotional mood
    pub mood: Mood,

    /// Duration in seconds (user-friendly)
    pub duration_seconds: f32,

    /// Intensity level
    #[serde(default = "default_intensity")]
    pub intensity: Intensity,

    /// Optional seed for deterministic generation
    pub seed: Option<u64>,

    /// Should it loop seamlessly?
    #[serde(default)]
    pub loop_seamlessly: bool,
}

fn default_intensity() -> Intensity {
    Intensity::Medium
}

impl SimpleMusicRequest {
    /// Convert to TuneProgram
    pub fn to_tune_program(&self) -> Result<TuneProgram> {
        let profile = MoodProfile::for_mood(self.mood);
        let mut rng = StdRng::seed_from_u64(self.seed.unwrap_or_else(|| rand::random()));

        let tempo = profile.random_tempo(&mut rng);
        let bars = self.calculate_bars(tempo);

        Ok(TuneProgram {
            version: 1,
            description: format!("{:?} mood music", self.mood),
            style: profile.style.to_string(),
            tempo,
            time_signature: (4, 4),
            key: profile.random_key(&mut rng),
            energy: profile.random_energy(&mut rng, self.intensity),
            duration_bars: bars,
            seed: self.seed,
            instruments: profile.instruments.clone(),
        })
    }

    /// Calculate number of bars from duration and tempo
    fn calculate_bars(&self, tempo: u32) -> u32 {
        // bars = (seconds * BPM) / (60 * beats_per_bar)
        let beats_per_bar = 4.0; // Assuming 4/4 time
        let total_beats = (self.duration_seconds * tempo as f32) / 60.0;
        let bars = (total_beats / beats_per_bar).ceil() as u32;

        // Ensure at least 2 bars, max 64 for performance
        bars.max(2).min(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mood_profile_happy() {
        let profile = MoodProfile::for_mood(Mood::Happy);
        assert_eq!(profile.mood, Mood::Happy);
        assert!(profile.tempo_range.0 >= 120);
        assert!(!profile.use_dissonance);
    }

    #[test]
    fn test_mood_profile_suspense() {
        let profile = MoodProfile::for_mood(Mood::Suspense);
        assert_eq!(profile.mood, Mood::Suspense);
        assert!(profile.use_dissonance);
        assert_eq!(profile.rhythm_pattern, RhythmPattern::Irregular);
    }

    #[test]
    fn test_calculate_bars() {
        let request = SimpleMusicRequest {
            mood: Mood::Happy,
            duration_seconds: 30.0,
            intensity: Intensity::Medium,
            seed: Some(42),
            loop_seamlessly: false,
        };

        // At 120 BPM: 30 seconds = 60 beats = 15 bars
        let bars = request.calculate_bars(120);
        assert_eq!(bars, 15);
    }

    #[test]
    fn test_simple_request_to_program() {
        let request = SimpleMusicRequest {
            mood: Mood::Calm,
            duration_seconds: 20.0,
            intensity: Intensity::Light,
            seed: Some(123),
            loop_seamlessly: false,
        };

        let program = request.to_tune_program().unwrap();
        assert_eq!(program.style, "ambient");
        assert!(program.energy < 0.5);
    }

    #[test]
    fn test_intensity_energy_ranges() {
        assert_eq!(Intensity::Light.energy_range(), (0.2, 0.4));
        assert_eq!(Intensity::Medium.energy_range(), (0.4, 0.6));
        assert_eq!(Intensity::Intense.energy_range(), (0.6, 0.9));
    }

    #[test]
    fn test_all_moods_have_profiles() {
        for mood in Mood::all() {
            let profile = MoodProfile::for_mood(mood);
            assert!(!profile.instruments.is_empty());
            assert!(profile.tempo_range.0 < profile.tempo_range.1);
        }
    }
}
