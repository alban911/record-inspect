# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.1.1] - 2026-11-07

### Added
- **Streaming DSP Pipe (`src/engine.rs`)**: Stream PCM blocks through a fixed-size buffer (16 KB / 3-second window) via `ChildStdout`, enabling analysis of indefinite audio lengths with constant ~100 KB resident memory.
- **Pre-Compiled Offline CSS**: Pre-compiled all 207 Tailwind CSS utility classes into static inline CSS inside `src/ui.html`, completely eliminating external CDN script and web font dependencies.
- **Robust Font Fallback**: Offline native typography stacks for Monospace (`JetBrains Mono`, `Cascadia Code`, `Consolas`) and Swiss UI (`Plus Jakarta Sans`, system sans-serif).
- **Lossy Audio Bitrate Extraction**: Extract and display actual bitrates (e.g. `320 kbps`) for MP3, AAC, Vorbis, and Opus files.
- **Automated Release Notes**: Integrated `RELEASE_NOTES.md` body delivery in `.github/workflows/release.yml`.

### Changed
- **Unified Engine Core**: Refactored `src/main.rs` to directly call `record_inspect_lib::analyze_track_data` and `print_terminal_report()`, eliminating redundant FFmpeg analysis code and preventing DSP logic drift.
- **Lossy Audio Bit Depth Representation**: Clarified that lossy perceptual codecs do not possess fixed physical PCM bit depths; replaced misleading `24-bit` and `31-bit` outputs with `Lossy Perceptual / IMDCT Float Decode`.
- **Windows MinGW Build Path**: Redirected `app_icon.o` build output to the ASCII `OUT_DIR` to prevent GNU `ld.exe` from failing on paths with non-ASCII or Chinese characters.

### Fixed
- Fixed an issue where MP3 files transcoded from 16-bit CDs were misidentified as 24-bit PCM due to FFmpeg's `fltp` decoder output.
- Fixed an issue where `astats` float mantissa precision was misreported as `31-bit` active bit depth on lossy tracks.
- Removed `-t 1800` (30-minute) hard cutoff in PCM decoding.

---

## [v0.1.0] - 2026-11-05

### Added
- Initial public open-source release of Record Inspect.
- High-precision acoustic metrology engine supporting EBU R128 LUFS, LRA, 4x Oversampled True Peak, Crest Factor, TT Dynamic Range (DR), and Phase Correlation.
- Dual-theme native desktop GUI (Swiss Anodized Aluminum and Brutalist Studio Rack) powered by Tao and Wry.
- CLI batch analyzer and AI Agent Blind-Testing Skill (`skill/SKILL.md`).
- GitHub Actions CI workflow for automated Windows x64 release packaging.
