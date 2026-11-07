# Record Inspect v0.1.1 Release Notes

## 🚀 What's New in v0.1.1

### 1. 🌊 Streaming DSP Pipeline (PCM 管道流式分块计算)
- **Zero-OOM Guarantee**: Refactored the core metrology engine (`src/engine.rs`) from buffering the entire uncompressed PCM in memory (`Command::output()`) to a piped streaming reader (`ChildStdout`) with fixed 16 KB chunk buffers and 3-second sliding statistics.
- **Removed Duration Limit**: Completely eliminated the previous `-t 1800` (30-minute) hard cutoff. Multi-hour symphonies, full-length operas, and vinyl rips now analyze with a constant, minimal resident memory footprint under 100 KB.

### 2. 📴 100% Offline & Self-Contained GUI
- **Zero External Network Dependencies**: Eliminated external script tags (`tailwindcss.min.js`) and Google Fonts `@import` network requests in `src/ui.html`.
- **Pre-Compiled Inlined CSS**: Inlined 207 utility CSS rules directly into the binary, achieving instant cold starts in air-gapped / offline environments and eliminating CDN supply chain exposure.
- **High-Precision Font Stacks**: Added robust fallback typography stacks for both Monospace terminal data (JetBrains Mono, Cascadia Code, Consolas) and Swiss UI (Plus Jakarta Sans, system sans-serif).

### 3. 🎯 Unified Single-Source Metrology Core
- **Eliminated Code Drift**: Refactored `src/main.rs` (CLI) to directly consume `record_inspect_lib::analyze_track_data`, shrinking CLI footprint by ~70% and ensuring 100% mathematical consistency across GUI, CLI, and AI Agent Skill workflows.

### 4. 🎛️ Lossy Codec Precision & Bit Depth Correction (MP3/AAC/OGG)
- **Acoustic Reality Alignment**: Fixed an issue where lossy perceptual formats (e.g., MP3 encoded from 16-bit Red Book CDs) were misidentified as 24-bit PCM due to FFmpeg's default `fltp` (float planar) decoder output.
- **Dynamic Bitrate Extraction**: Lossy files now display actual stream bitrates (e.g., `320 kbps (Lossy Perceptual)`) rather than misleading physical PCM bit depths.
- **Accurate Active Depth Labeling**: Corrected `astats` float mantissa misinterpretation (which previously reported 31-bit) to explicitly indicate perceptual dynamic quantization.

### 5. 🛠️ Windows MinGW Build Hardening
- **Path Sanitization**: Fixed an issue in `build.rs` where GNU `windres` / `ld.exe` failed to locate icon object files when the workspace directory contained non-ASCII / Chinese characters. Object artifacts now output cleanly to the ASCII `OUT_DIR`.

---

### Verification Summary
- **Windows x64 Native Binaries**: Built with Rust 2021 GNU toolchain with full LTO and stripped symbols (`gui.exe`: 1.07 MB, `cli.exe`: 606 KB).
- **Tested Audio Codecs**: FLAC (16/44.1, 24/96, 24/192), WAV (PCM & 32-bit float), MP3 (320 kbps CBR & VBR), DSD (.dsf / .dff DSD64).
