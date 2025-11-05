//! Metadata tagging and library management for generated music
//!
//! This module provides ID3-like metadata tagging for generated music,
//! enabling organization, searchability, and reuse of compositions.

use crate::{Result, Error, TuneProgram, mood::{Mood, Intensity}};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

/// Comprehensive metadata for a generated music track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicMetadata {
    /// Unique identifier (hash of parameters)
    pub id: String,

    /// Primary mood tag
    pub mood: Mood,

    /// Intensity level
    pub intensity: Intensity,

    /// Duration in seconds
    pub duration_seconds: f32,

    /// Tempo (BPM)
    pub tempo: u32,

    /// Key signature
    pub key: String,

    /// Musical style/genre
    pub style: String,

    /// Energy level (0.0-1.0)
    pub energy: f32,

    /// Instruments used
    pub instruments: Vec<String>,

    /// Generation seed (for reproducibility)
    pub seed: u64,

    /// Timestamp when generated
    pub created_at: SystemTime,

    /// File paths for different formats
    pub files: MusicFiles,

    /// User-defined tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Use cases (from mood profile)
    #[serde(default)]
    pub use_cases: Vec<String>,

    /// Number of times used
    #[serde(default)]
    pub usage_count: u32,

    /// Last used timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<SystemTime>,
}

/// File paths for generated music in different formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicFiles {
    /// MIDI file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi: Option<PathBuf>,

    /// WAV mix file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wav: Option<PathBuf>,

    /// GarageBand bundle path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub garageband: Option<PathBuf>,
}

impl MusicMetadata {
    /// Create metadata from a tune program
    pub fn from_program(
        program: &TuneProgram,
        mood: Mood,
        intensity: Intensity,
        duration_seconds: f32,
        seed: u64,
    ) -> Self {
        Self {
            id: Self::generate_id(mood, intensity, duration_seconds, program.tempo),
            mood,
            intensity,
            duration_seconds,
            tempo: program.tempo,
            key: program.key.clone(),
            style: program.style.clone(),
            energy: program.energy,
            instruments: program.instruments.iter().map(|i| format!("{:?}", i)).collect(),
            seed,
            created_at: SystemTime::now(),
            files: MusicFiles {
                midi: None,
                wav: None,
                garageband: None,
            },
            tags: Vec::new(),
            use_cases: mood.use_cases().iter().map(|s| s.to_string()).collect(),
            usage_count: 0,
            last_used: None,
        }
    }

    /// Generate a unique ID from key parameters
    fn generate_id(mood: Mood, intensity: Intensity, duration: f32, tempo: u32) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}_{:?}_{:.1}_{}", mood, intensity, duration, tempo).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Add a custom tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Record usage
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(SystemTime::now());
    }

    /// Check if this matches search criteria
    pub fn matches(&self, query: &MusicQuery) -> bool {
        // Mood match
        if let Some(mood) = query.mood {
            if self.mood != mood {
                return false;
            }
        }

        // Intensity match
        if let Some(intensity) = query.intensity {
            if self.intensity != intensity {
                return false;
            }
        }

        // Duration range
        if let Some((min, max)) = query.duration_range {
            if self.duration_seconds < min || self.duration_seconds > max {
                return false;
            }
        }

        // Style match
        if let Some(ref style) = query.style {
            if !self.style.contains(style) {
                return false;
            }
        }

        // Tag match
        if let Some(ref tag) = query.tag {
            if !self.tags.iter().any(|t| t.contains(tag)) {
                return false;
            }
        }

        // Use case match
        if let Some(ref use_case) = query.use_case {
            if !self.use_cases.iter().any(|uc| uc.to_lowercase().contains(&use_case.to_lowercase())) {
                return false;
            }
        }

        true
    }
}

/// Query for searching music library
#[derive(Debug, Clone, Default)]
pub struct MusicQuery {
    pub mood: Option<Mood>,
    pub intensity: Option<Intensity>,
    pub duration_range: Option<(f32, f32)>,
    pub style: Option<String>,
    pub tag: Option<String>,
    pub use_case: Option<String>,
}

/// Music library for managing generated tracks
pub struct MusicLibrary {
    /// Root directory for library storage
    root_path: PathBuf,

    /// In-memory metadata index
    index: Vec<MusicMetadata>,
}

impl MusicLibrary {
    /// Create or open a music library
    pub fn new(root_path: PathBuf) -> Result<Self> {
        // Create directory structure
        fs::create_dir_all(&root_path)
            .map_err(|e| Error::export(format!("Failed to create library directory: {}", e)))?;

        fs::create_dir_all(root_path.join("midi"))
            .map_err(|e| Error::export(format!("Failed to create MIDI directory: {}", e)))?;

        fs::create_dir_all(root_path.join("wav"))
            .map_err(|e| Error::export(format!("Failed to create WAV directory: {}", e)))?;

        fs::create_dir_all(root_path.join("garageband"))
            .map_err(|e| Error::export(format!("Failed to create GarageBand directory: {}", e)))?;

        // Load existing index
        let mut library = Self {
            root_path,
            index: Vec::new(),
        };

        library.load_index()?;
        Ok(library)
    }

    /// Get library path
    pub fn path(&self) -> &Path {
        &self.root_path
    }

    /// Get index file path
    fn index_path(&self) -> PathBuf {
        self.root_path.join("library_index.json")
    }

    /// Load index from disk
    fn load_index(&mut self) -> Result<()> {
        let index_path = self.index_path();
        if index_path.exists() {
            let data = fs::read_to_string(&index_path)
                .map_err(|e| Error::export(format!("Failed to read index: {}", e)))?;

            self.index = serde_json::from_str(&data)
                .map_err(|e| Error::export(format!("Failed to parse index: {}", e)))?;

            tracing::info!("Loaded library with {} tracks", self.index.len());
        }
        Ok(())
    }

    /// Save index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.index_path();
        let data = serde_json::to_string_pretty(&self.index)
            .map_err(|e| Error::export(format!("Failed to serialize index: {}", e)))?;

        fs::write(&index_path, data)
            .map_err(|e| Error::export(format!("Failed to write index: {}", e)))?;

        tracing::debug!("Saved library index with {} tracks", self.index.len());
        Ok(())
    }

    /// Add a track to the library
    pub fn add_track(&mut self, mut metadata: MusicMetadata) -> Result<String> {
        // Check if similar track already exists
        if let Some(existing) = self.find_similar(&metadata) {
            tracing::info!("Similar track already exists: {}", existing.id);
            return Ok(existing.id.clone());
        }

        // Ensure unique ID
        let mut counter = 1;
        let base_id = metadata.id.clone();
        while self.index.iter().any(|m| m.id == metadata.id) {
            metadata.id = format!("{}_{}", base_id, counter);
            counter += 1;
        }

        let id = metadata.id.clone();
        self.index.push(metadata);
        self.save_index()?;

        tracing::info!("Added track to library: {}", id);
        Ok(id)
    }

    /// Find a track by ID
    pub fn get_track(&self, id: &str) -> Option<&MusicMetadata> {
        self.index.iter().find(|m| m.id == id)
    }

    /// Find a track by ID (mutable)
    pub fn get_track_mut(&mut self, id: &str) -> Option<&mut MusicMetadata> {
        self.index.iter_mut().find(|m| m.id == id)
    }

    /// Search library with query
    pub fn search(&self, query: &MusicQuery) -> Vec<&MusicMetadata> {
        self.index.iter()
            .filter(|m| m.matches(query))
            .collect()
    }

    /// Find similar track (within 10% duration, same mood/intensity)
    fn find_similar(&self, metadata: &MusicMetadata) -> Option<&MusicMetadata> {
        let duration_tolerance = metadata.duration_seconds * 0.1;

        self.index.iter().find(|m| {
            m.mood == metadata.mood &&
            m.intensity == metadata.intensity &&
            (m.duration_seconds - metadata.duration_seconds).abs() < duration_tolerance
        })
    }

    /// Get most used tracks
    pub fn get_popular(&self, limit: usize) -> Vec<&MusicMetadata> {
        let mut sorted = self.index.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        sorted.into_iter().take(limit).collect()
    }

    /// Get recently generated tracks
    pub fn get_recent(&self, limit: usize) -> Vec<&MusicMetadata> {
        let mut sorted = self.index.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted.into_iter().take(limit).collect()
    }

    /// Get all moods available in library
    pub fn get_available_moods(&self) -> Vec<Mood> {
        use std::collections::HashSet;
        let moods: HashSet<Mood> = self.index.iter().map(|m| m.mood).collect();
        let mut result: Vec<_> = moods.into_iter().collect();
        result.sort_by_key(|m| format!("{:?}", m));
        result
    }

    /// Get library statistics
    pub fn stats(&self) -> LibraryStats {
        let total_tracks = self.index.len();
        let total_duration: f32 = self.index.iter().map(|m| m.duration_seconds).sum();
        let total_usage: u32 = self.index.iter().map(|m| m.usage_count).sum();

        LibraryStats {
            total_tracks,
            total_duration_seconds: total_duration,
            total_usage,
            moods: self.get_available_moods().len(),
        }
    }

    /// Update track files
    pub fn update_files(&mut self, id: &str, files: MusicFiles) -> Result<()> {
        if let Some(track) = self.get_track_mut(id) {
            track.files = files;
            self.save_index()?;
            Ok(())
        } else {
            Err(Error::NotFound(format!("Track not found: {}", id)))
        }
    }

    /// Record track usage
    pub fn record_usage(&mut self, id: &str) -> Result<()> {
        if let Some(track) = self.get_track_mut(id) {
            track.record_usage();
            self.save_index()?;
            Ok(())
        } else {
            Err(Error::NotFound(format!("Track not found: {}", id)))
        }
    }

    /// Delete a track from library
    pub fn delete_track(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.index.iter().position(|m| m.id == id) {
            let metadata = self.index.remove(pos);

            // Delete associated files
            if let Some(midi) = metadata.files.midi {
                let _ = fs::remove_file(midi);
            }
            if let Some(wav) = metadata.files.wav {
                let _ = fs::remove_file(wav);
            }
            if let Some(gb) = metadata.files.garageband {
                let _ = fs::remove_dir_all(gb);
            }

            self.save_index()?;
            tracing::info!("Deleted track: {}", id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Track not found: {}", id)))
        }
    }
}

/// Library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_tracks: usize,
    pub total_duration_seconds: f32,
    pub total_usage: u32,
    pub moods: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Instrument;

    fn create_test_metadata() -> MusicMetadata {
        let program = TuneProgram {
            version: 1,
            description: "Test".into(),
            style: "pop".into(),
            tempo: 120,
            time_signature: (4, 4),
            key: "C major".into(),
            energy: 0.7,
            duration_bars: 8,
            seed: Some(42),
            instruments: vec![Instrument::Piano],
        };

        MusicMetadata::from_program(&program, Mood::Happy, Intensity::Medium, 30.0, 42)
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.mood, Mood::Happy);
        assert_eq!(metadata.intensity, Intensity::Medium);
        assert_eq!(metadata.duration_seconds, 30.0);
    }

    #[test]
    fn test_metadata_matching() {
        let metadata = create_test_metadata();

        let query = MusicQuery {
            mood: Some(Mood::Happy),
            ..Default::default()
        };
        assert!(metadata.matches(&query));

        let query = MusicQuery {
            mood: Some(Mood::Sad),
            ..Default::default()
        };
        assert!(!metadata.matches(&query));
    }

    #[test]
    fn test_library_creation() {
        let temp_dir = std::env::temp_dir().join("tunesmith_test_library");
        let library = MusicLibrary::new(temp_dir.clone());
        assert!(library.is_ok());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_library_add_and_search() {
        let temp_dir = std::env::temp_dir().join("tunesmith_test_library_2");
        let mut library = MusicLibrary::new(temp_dir.clone()).unwrap();

        let metadata = create_test_metadata();
        let id = library.add_track(metadata).unwrap();

        let found = library.get_track(&id);
        assert!(found.is_some());

        let query = MusicQuery {
            mood: Some(Mood::Happy),
            ..Default::default()
        };
        let results = library.search(&query);
        assert_eq!(results.len(), 1);

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
