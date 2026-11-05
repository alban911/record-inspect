# Record Inspect

[![Release](https://github.com/alban911/record-inspect/actions/workflows/release.yml/badge.svg)](https://github.com/alban911/record-inspect/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows_x64-lightgrey.svg)]()

> **High-Precision Acoustic Metrology, Mastering Quality Inspection & AI Blind-Testing Protocol.**
>
> A native toolchain engineered for audio engineers, audiophiles, and mastering quality control: zero-dependency Rust DSP engine + dual-theme industrial GUI + batch CLI + AI Agent blind-testing skill.

[English](README.md) | [简体中文](README_zh.md)

---

## Core Features

- **High-Performance Rust Metrology Architecture (`src/engine.rs`)**
  - Ultra-fast zero-copy DSP analysis pipeline with zero Python runtime overhead.
  - Leverages system-level **FFmpeg** for universal audio decoding (FLAC, WAV, APE, ALAC, MP3, and native DSD `.dsf`/`.dff`).
  - Strictly adheres to industrial standards (ITU-R BS.1770-4, EBU R128).
- **Comprehensive Acoustic Metrology**
  - **Dynamics & Loudness**: EBU R128 Integrated Loudness (`LUFS`), Loudness Range (`LRA`), 4x Oversampled True Peak (`dBTP`), Crest Factor (`dB`), and classic `TT Dynamic Range (DR)`.
  - **Stereo Field & Phase**: Full-band Phase Correlation Coefficient (`-1.0 ~ +1.0`), Side/Mid Energy Ratio (`dB`).
  - **Cutoff Frequency & Fake Hi-Res Detection**: High-resolution spectral rolloff detection identifying brickwall low-pass filters at 16 kHz, 20 kHz, 22.05 kHz, or 24 kHz (unmasking MP3 transcode upscales and fake 96k/192k releases).
  - **Bit Depth & DC Drift**: Active Bit Depth histogram analysis and DC Offset detection.
- **Dual-Persona Native GUI (`record-inspect-gui`)**
  - Lightweight native desktop client powered by Tao + Wry (WebView2).
  - **Swiss Anodized Aluminum**: Minimalist, clean, high-contrast precision instrument aesthetic.
  - **Brutalist Studio Rack**: Industrial physical rack unit styling with tactile toggles and LED indicators.
  - Drag-and-drop instant measurement with multi-file tabular inspection.
- **Headless Command-Line Interface (`record-inspect-cli`)**
  - Single-track and recursive directory analysis emitting formatted terminal reports or structured JSON.
- **AI Agent Blind-Testing Skill (`skill/`)**
  - Standardized LLM prompt framework for objective mastering evaluations.
  - Enforces blind testing by omitting song title, artist, and label metadata—critiques playback load and acoustic master quality strictly using physical metrics and user-provided musical instrumentation.

---

## Repository Structure

```text
record-inspect/
├── .github/
│   └── workflows/
│       └── release.yml          # Automated CI/CD release workflow
├── assets/                      # Application icons, vector SVGs, and Win32 RC
│   ├── app.rc
│   ├── app_icon.ico
│   └── ...
├── docs/
│   └── METRICS_EXPLANATION.md   # Detailed acoustics & metrics reference manual
├── scripts/                     # Python scripts (batch runner, docx export)
│   ├── batch_analyzer.py
│   ├── export_docx.py
│   └── record_inspect.py
├── skill/                       # AI Agent Skill specification and templates
│   ├── templates/
│   │   ├── album_summary.md
│   │   └── single_track.md
│   └── SKILL.md
├── src/                         # Rust source code
│   ├── engine.rs                # DSP engine and measurement data structures
│   ├── gui_main.rs              # Native desktop GUI entry point
│   ├── main.rs                  # Command-line interface entry point
│   └── ui.html                  # Dual-theme GUI HTML/CSS/JS template
├── Cargo.toml
├── LICENSE
├── README.md
└── README_zh.md
```

---

## Prerequisites

Both `record-inspect-gui` and `record-inspect-cli` require **FFmpeg** to be installed and available in your system's `PATH`:

- **Windows Installation (Scoop / Winget)**:
  ```powershell
  # Using Scoop
  scoop install ffmpeg

  # Or using Winget
  winget install Gyan.FFmpeg
  ```
- **Verification**: Run `ffmpeg -version` in your terminal to ensure it is recognized.

---

## Quick Start

### 1. Prebuilt Binaries
Download the latest Windows x64 standalone executables from the [Releases page](https://github.com/alban911/record-inspect/releases):
- `record-inspect-gui.exe`: Standalone desktop graphical application.
- `record-inspect-cli.exe`: Command-line interface for batch processing.

### 2. Building from Source
Ensure Rust 1.75+ is installed:

```bash
# Clone the repository
git clone https://github.com/alban911/record-inspect.git
cd record-inspect

# Build optimized release binaries
cargo build --release

# Binaries located at target/release/:
# - record-inspect-gui.exe
# - record-inspect-cli.exe
```

---

## Usage

### Native GUI Application
Launch `record-inspect-gui.exe` and drop audio files (`.wav`, `.flac`, `.aiff`, etc.) directly into the window. Metrics are computed and displayed in real time:
- Toggle between **Swiss Aluminum** and **Brutalist Rack** themes at the top bar.
- Critical risks (such as True Peak clipping or fake Hi-Res cutoff) are automatically flagged in red/amber.

### Command-Line Interface (CLI)

```bash
# Analyze a single audio file
record-inspect-cli.exe "path/to/track.flac"

# Output structured JSON
record-inspect-cli.exe --json "path/to/track.flac"

# Batch analyze an entire directory
record-inspect-cli.exe "D:/Music/Master_Files"
```

### AI Agent Skill Integration
Integrate the `skill/` directory into your agentic workflow (e.g. Antigravity, Claude Projects, Custom GPTs):
1. Extract audio metrics via `record-inspect-cli.exe --json`.
2. Provide the metrics payload alongside the genre/instrumentation (e.g., *Full Symphonic Orchestra*).
3. The model performs an objective blind-test review based on physical audio properties, avoiding subjective buzzwords.

---

## Acoustic Metrics Summary

| Metric | Reference / Unit | Recommended Range | Defect / Risk Threshold |
| :--- | :--- | :--- | :--- |
| **Integrated Loudness** | EBU R128 (`LUFS`) | Classical: -24 ~ -18 / Pop: -14 ~ -9 | > -7 LUFS (Heavy dynamic squashing) |
| **Loudness Range (LRA)**| EBU R128 (`LU`) | Symphony: > 14 / Pop: 6 ~ 10 | < 4 LU (Near-squarewave brickwalling) |
| **True Peak (TP)** | ITU-R BS.1770 (`dBTP`)| $\le -1.0\text{ dBTP}$ | $> 0.0\text{ dBTP}$ (DAC inter-sample clip) |
| **TT Dynamic Range** | `DR` Rating | DR10 ~ DR16 (Natural dynamic contrast) | $\le \text{DR6}$ (Excessive peak limiting) |
| **Crest Factor** | Transient Ratio (`dB`)| 12 ~ 18 dB | < 10 dB (Transient attack erosion) |
| **Phase Correlation** | Coefficient (`-1 ~ +1`)| +0.6 ~ +0.95 (Mono-compatible stereo) | < 0 (Phase cancellation risk) |
| **Cutoff Frequency** | Rolloff Edge (`kHz`) | 44.1k/48k: Unfiltered / 96k+: > 40k | Rolloff at 20k/22.05k (Fake Hi-Res) |

*For mathematical definitions and acoustic explanations, refer to [docs/METRICS_EXPLANATION.md](docs/METRICS_EXPLANATION.md).*

---

## License

This project is licensed under the [MIT License](LICENSE).
