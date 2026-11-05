#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
batch_analyzer.py - Record Inspector Batch & Single File Acoustic Extractor
Part of the record-inspector skill for Antigravity / Obsidian
"""

import sys
import os
import re
import json
import shutil
import subprocess
from pathlib import Path

SUPPORTED_EXTS = {".flac", ".wav", ".dsf", ".dff", ".aiff", ".aif", ".m4a", ".mp3", ".ape", ".wv"}

SCRIPT_DIR = Path(__file__).resolve().parent
SKILL_DIR = SCRIPT_DIR.parent
PY_SCRIPT = SCRIPT_DIR / "record_inspect.py"

def find_cli_binary() -> Path | None:
    """Auto-discover the native Rust CLI binary across common locations and PATH."""
    candidates = [
        SKILL_DIR / "record-inspect-cli.exe",
        SKILL_DIR / "record-inspect.exe",
        SKILL_DIR / "bin" / "record-inspect-cli.exe",
        SKILL_DIR / "bin" / "record-inspect.exe",
        SKILL_DIR / "target" / "release" / "record-inspect-cli.exe",
        SCRIPT_DIR / "record-inspect-cli.exe",
    ]
    for c in candidates:
        if c.is_file():
            return c
    for cmd in ("record-inspect-cli", "record-inspect"):
        which_path = shutil.which(cmd)
        if which_path:
            return Path(which_path)
    return None

def run_inspector(audio_file: str) -> dict:
    """Run native Rust CLI binary (preferred) or record_inspect.py to extract metrics."""
    audio_path = Path(audio_file).resolve()
    if not audio_path.exists():
        return {"error": f"File not found: {audio_file}"}

    raw_output = ""
    # 1. Try auto-discovered Rust CLI binary
    bin_exe = find_cli_binary()
    if bin_exe:
        try:
            p = subprocess.run([str(bin_exe), str(audio_path)], capture_output=True, text=True, encoding="utf-8", errors="replace")
            if p.returncode == 0:
                raw_output = p.stdout
        except Exception:
            pass

    # 2. Fallback to Python
    if not raw_output and PY_SCRIPT.exists():
        try:
            p = subprocess.run([sys.executable, str(PY_SCRIPT), str(audio_path)], capture_output=True, text=True, encoding="utf-8", errors="replace")
            if p.returncode == 0:
                raw_output = p.stdout
        except Exception as e:
            return {"error": f"Execution failed: {e}"}

    if not raw_output:
        return {"error": "Failed to extract metrics from audio engine."}

    # Parse metrics from stdout
    metrics = {
        "file_name": audio_path.name,
        "file_path": str(audio_path),
        "file_size_mb": round(audio_path.stat().st_size / (1024 * 1024), 2),
        "raw_text": raw_output
    }

    patterns = {
        "format": r"格式规格:\s*([^\n|]+)",
        "sample_rate": r"(\d+)\s*Hz",
        "bit_depth": r"(\d+)-bit",
        "duration": r"时长:\s*([0-9:]+)",
        "lufs": r"综合响度.*?:\s*([+-]?\d+\.?\d*)\s*LUFS",
        "lra": r"宏观响度范围.*?:\s*([+-]?\d+\.?\d*)\s*LU",
        "true_peak": r"最高真峰值.*?:\s*([+-]?\d+\.?\d*)\s*dBFS",
        "overall_rms": r"平均有效值.*?:\s*([+-]?\d+\.?\d*)\s*dBFS",
        "crest": r"瞬态峰值因子.*?:\s*([+-]?\d+\.?\d*)\s*dB",
        "dr_score": r"TT DR 动态评分.*?:\s*(DR\d+)",
        "dr_num": r"TT DR 动态评分.*?\(\s*([0-9.]+)\s*dB\)",
        "loudest_3s": r"最响 3 秒.*?:\s*([+-]?\d+\.?\d*)\s*dBFS\s*\((.*?)\)",
        "quietest_3s": r"最弱 3 秒.*?:\s*([+-]?\d+\.?\d*)\s*dBFS\s*\((.*?)\)",
        "macro_span": r"宏观动态极差.*?:\s*([+-]?\d+\.?\d*)\s*dB",
        "corr": r"全频双声道相关性.*?:\s*([+-]?\d+\.?\d*)",
        "sm_ratio": r"中/侧能量比.*?:\s*([+-]?\d+\.?\d*)\s*dB",
        "balance": r"左右声道重心.*?:\s*([+-]?\d+\.?\d*)\s*dB",
        "low_corr": r"低频段.*?Corr:\s*([+-]?\d+\.?\d*)",
        "mid_corr": r"中频段.*?Corr:\s*([+-]?\d+\.?\d*)",
        "high_corr": r"高频段.*?Corr:\s*([+-]?\d+\.?\d*)",
        "cutoff": r"实测有效截止频率.*?:\s*([0-9.]+)\s*kHz",
        "band10k": r"10kHz - 20kHz.*?:\s*([+-]?\d+\.?\d*)\s*dBFS",
        "band20k": r"20kHz - 24kHz.*?:\s*([+-]?\d+\.?\d*)\s*dBFS",
        "band24k": r"24kHz - 48kHz.*?:\s*([+-]?\d+\.?\d*)\s*dBFS",
        "dc_offset": r"直流偏移.*?L:\s*([+-]?\d+\.?\d*),\s*R:\s*([+-]?\d+\.?\d*)",
        "active_bits": r"有效利用位深.*?:\s*(\d+)-bit"
    }

    for k, pat in patterns.items():
        m = re.search(pat, raw_output)
        if m:
            if len(m.groups()) == 1:
                metrics[k] = m.group(1).strip()
            elif len(m.groups()) == 2:
                metrics[k] = (m.group(1).strip(), m.group(2).strip())

    return metrics

def scan_target(target_path: str) -> dict:
    p = Path(target_path).resolve()
    if not p.exists():
        return {"error": f"Target path does not exist: {target_path}"}

    if p.is_file():
        # Single track analysis
        res = run_inspector(str(p))
        return {"mode": "single", "data": res}
    elif p.is_dir():
        # Album / Batch analysis
        tracks = []
        for file in sorted(p.iterdir()):
            if file.suffix.lower() in SUPPORTED_EXTS:
                tracks.append(str(file))

        if not tracks:
            return {"error": f"No supported audio tracks found in directory: {target_path}"}

        results = []
        for track in tracks:
            r = run_inspector(track)
            if "error" not in r:
                results.append(r)

        return {
            "mode": "album",
            "album_name": p.name,
            "album_path": str(p),
            "track_count": len(results),
            "tracks": results
        }
    else:
        return {"error": f"Unknown target type: {target_path}"}

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: batch_analyzer.py <audio_file_or_directory>"}, ensure_ascii=False))
        sys.exit(1)

    target = sys.argv[1]
    res = scan_target(target)
    print(json.dumps(res, ensure_ascii=False, indent=2))

if __name__ == "__main__":
    main()
