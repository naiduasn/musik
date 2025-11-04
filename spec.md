# Revised Engineering Specification: TuneSmith Local

## 1. Introduction
This revised engineering specification incorporates feedback from the review, addressing risks such as licensing, complexity scoping, performance, and missing deliverables. We maintain a no-compromise approach to quality, prioritizing Rust's safety and efficiency, while enhancing developer-friendliness with clearer APIs, phased features, and mitigated risks. Key updates include switching to a pure-Rust, MIT-licensed synthesizer (RustySynth) to avoid LGPL issues, phasing VO processing, enforcing ONNX-only inference for consistency, and adding sections on concurrency, testing, and IPC APIs. The MVP is streamlined to 3 core features for faster delivery (6-8 weeks).

Principles remain: Local-first, deterministic, performant, modular, secure.

## 2. Architecture Overview
Unchanged core structure: Rust backend + Tauri (2.x) frontend with Svelte (5.x). Data flow as before.

Updates:
- **Synthesis Switch**: Replace fluidsynth with RustySynth (pure Rust, MIT/Apache licensed, supports SF2/SFZ). This eliminates LGPL concerns, enables static linking without restrictions, and keeps performance high (SIMD-optimized).
- **Model Inference**: Default to ONNX Runtime only for cross-platform consistency; Core ML as optional macOS accel with documented potential minor output deltas (e.g., floating-point precision variances). Accept deltas <0.1% in benchmarks or fallback to ONNX.
- **Modular Packs**: Soundfonts and optional models downloaded post-install via in-app (opt-in HTTPS from self-hosted CDN), reducing base binary to <100MB.
- **Phased VO**: MVP: Gate + ducking + simple EQ/limiter. M1: RNNoise + de-esser. M2: Full chain.
- **Error Handling**: Add audio recovery watchdog (monitor cpal stream; restart on underrun/freezes >500ms). Use tracing for logs with UI-toggleable levels (info/debug). Opt-in panic capture via sentry-rs (anonymized, no audio data).
- **GarageBand Integration**: Primary: Generate .band bundle (folder structure with ProjectData plist/XML, media stems, MIDI). Fallback: AppleScript/Shortcuts for automation. .band generation via reverse-engineered format (bundle with Media folder for audio/MIDI, DisplayState.plist for track metadata—tested for compatibility).

## 3. Tech Stack
Updates for risks:
- **Synthesis/Sampling**: RustySynth for SF2/SFZ (MIT/Apache, pure Rust, low-latency). Fallback to fundsp for basic VA synths if needed.
- **Model Runtime**: ONNX Runtime primary (cross-platform, deterministic). Quantize to 4-bit (Q4_K_M via gguf/llama.cpp compat); expected RAM: ~1.5-2GB peak (lazy-loaded in background thread to avoid UI blocks). Rule-based fallback always available.
- **Audio and MIDI**: Unchanged (cpal, midly, dasp, fundsp, rubato, ffmpeg-rs).
- **Effects**: Phased as above.
- **Utilities**: Add sentry-rs for opt-in crash reporting; tracing-subscriber for logs.
- **Packaging**: Base binary <100MB; sound packs as separate downloads (e.g., 50MB genre packs).

## 4. Detailed Component Specifications

### 4.1 Prompt & Brief Layer
Unchanged, but add JSON schema (using schemars) for Tune Program validation. Version field mandatory (start at v1).

### 4.2 Composition Engine
Unchanged. Add testable submodules: 
- `composition::harmony`: Chord progression generator (isolated, pure functions).
- `composition::melody`: Motif builder with RNG injection.
- `composition::rhythm`: Pattern generator.
Each with unit tests (e.g., seed → expected MIDI output).

### 4.3 Rendering Engine
Updates:
- Synthesis: RustySynth for sampling; integrate via its API (load SFZ/SF2, render MIDI to audio buffers).
- FX Chain: MVP simplified (EQ/comp via dasp; limiter with LUFS).
- Rendering Modes: Add watchdog in realtime stream (Tokio task monitors buffer health).

### 4.4 Audio I/O and VO
Updates:
- Phased: MVP gate (threshold-based), ducking (fundsp sidechain), EQ/limiter. Defer RNNoise/de-esser.
- Concurrency: Dedicated Tokio runtime for audio threads (one for capture, one for playback/mixing). Use mpsc channels for buffer ownership (Arc<Mutex<>> for shared state minimal).

### 4.5 Exports and Integration
Updates:
- GarageBand: Primary export .band bundle:
  - Create folder: `project.band/`
  - Subdirs: `Media/` (stems WAV, MIDI files), `Output/` (mix), root: `ProjectData` (binary plist with track info, tempo; generate via plist-rs).
  - `DisplayState.plist`: XML for UI state (tracks, volumes).
  - User opens bundle directly in GarageBand.
- Fallback: AppleScript to import into new project.
- Plugins: Phase 2, MIDI-only generation (no audio render in-plugin to avoid sandbox/complexity). Use steinberg-vst-rs (VST3) and au-rs (AUv3); scope to <1 month.

### 4.6 SDK/CLI
Unchanged.

## 5. Performance Optimizations
Updates:
- RAM Policy: Lazy-load models (spawn Tokio task on first NL prompt). Monitor via sysinfo crate; warn if <4GB free.
- Profiling: Add benchmarks for VO phases.

## 6. Security, Privacy, and Accessibility
Updates: Add opt-in sentry for crashes (symbolicated locally first).

## 7. Development and Testing
New Sections:

### 7.1 Audio Threads and Concurrency Ownership
- **Threads**: Main (Tauri event loop), Audio I/O (cpal callbacks in dedicated thread), Inference (Tokio task pool), Rendering (Tokio for offline).
- **Ownership**: Audio buffers use RingBuf (crossbeam-deque) for producer-consumer. MIDI data immutable post-generation (Rc<>). Avoid globals; inject dependencies.

### 7.2 Testable Submodules
- Composition: Pure functions; fuzz-tested with proptest.
- Rendering: Golden audio tests (compare WAV hashes).
- VO: Simulate mic input with test buffers.
- E2E: Tauri spectator for UI flows.

### 7.3 Public API Surface at IPC Layer
Documented commands (Tauri invoke/handler):
- `generate_tune`: Input: TuneProgram JSON/str. Output: MIDI bytes, seed.
- `render_preview`: Input: MIDI bytes, config. Output: Stream handle (for realtime).
- `record_vo`: Input: Device ID, duration. Output: WAV bytes.
- `mix_and_duck`: Input: Music/VO bytes. Output: Mixed WAV.
- `export_project`: Input: Format (stems/midi/band). Output: File path.
- `load_model`: Async, for lazy init.
Each with error types (serde JSON).

## 8. Revised MVP (3 Features)
1. Prompt → Tune: Rule-based + optional LLM (ONNX-only).
2. Preview + VO: Record with basic gate/ducking.
3. Export: Stems/MIDI + .band bundle (or AppleScript).

Defers: Advanced VO (RNNoise), full FX, plugins, style packs.

## 9. Risks and Mitigations
Updates:
- Licensing: Mitigated with RustySynth (MIT).
- VO Scope: Phased to prevent explosion.
- Model RAM: 4-bit + lazy load; fallback rule-based.
- Plugin: Scoped MIDI-only.
- GarageBand: .band primary (feasible as bundle; test reverse-eng).
- ONNX/CoreML: ONNX default; deltas accepted/tested.

This revision makes the spec more robust, implementable, and developer-friendly while upholding excellence.
