use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn create_cmd(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn format_time(seconds: f64) -> String {
    let total = seconds as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    let len = bytes.len() / 4;
    let mut vec = Vec::with_capacity(len);
    for chunk in bytes.chunks_exact(4) {
        vec.push(f32::from_le_bytes(chunk.try_into().unwrap()));
    }
    vec
}

fn run_band_spatial(file_path: &str, filter: &str) -> (f64, f64, f64) {
    let output = create_cmd("ffmpeg")
        .args([
            "-v", "quiet", "-i", file_path, "-af", filter, "-f", "f32le", "-ac", "2", "-ar", "44100", "-",
        ])
        .output();

    let bytes = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return (0.0, 1.0, -99.0),
    };

    let s = bytes_to_f32_vec(&bytes);
    if s.is_empty() {
        return (0.0, 1.0, -99.0);
    }

    let mut sum_l2 = 0.0f64;
    let mut sum_r2 = 0.0f64;
    let mut sum_lr = 0.0f64;

    for chunk in s.chunks_exact(2) {
        let l = chunk[0] as f64;
        let r = chunk[1] as f64;
        sum_l2 += l * l;
        sum_r2 += r * r;
        sum_lr += l * r;
    }

    let denom = (sum_l2 * sum_r2).sqrt();
    let corr = if denom > 1e-12 { sum_lr / denom } else { 1.0 };
    let bal = 10.0 * ((sum_l2 + 1e-12) / (sum_r2 + 1e-12)).log10();
    let mid_e = (sum_l2 + sum_r2 + 2.0 * sum_lr) / 4.0;
    let side_e = (sum_l2 + sum_r2 - 2.0 * sum_lr) / 4.0;
    let sm = if mid_e > 1e-12 && side_e > 1e-12 {
        10.0 * (side_e / mid_e).log10()
    } else {
        -99.0
    };

    (bal, corr, sm)
}

fn run_hires_band(file_path: &str, filter: &str) -> f64 {
    let output = create_cmd("ffmpeg")
        .args(["-v", "quiet", "-i", file_path, "-af", filter, "-f", "f32le", "-ac", "2", "-"])
        .output();

    let bytes = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return -120.0,
    };

    let s = bytes_to_f32_vec(&bytes);
    if s.is_empty() {
        return -120.0;
    }

    let sum_sq: f64 = s.iter().map(|&x| (x as f64) * (x as f64)).sum();
    let rms = (sum_sq / s.len() as f64).sqrt();
    20.0 * (rms + 1e-12).log10()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub title: String,
    pub path: String,
    pub codec: String,
    pub spec: String,
    pub dur: String,
    pub duration_sec: f64,
    pub size_mb: f64,
    pub status: String,

    pub lufs: String,
    pub lufs_pct: f64,
    pub tp: String,
    pub tp_pct: f64,
    pub dr: String,
    pub dr_num: f64,
    pub crest: String,
    pub lra: String,
    pub span: String,

    pub cutoff: String,
    pub cutoff_tag: String,
    pub band10k: String,
    pub band20k: String,
    pub band24k: String,

    pub dc: String,
    pub bits: String,

    pub corr: String,
    pub sm: String,
    pub bal: String,
    pub low: String,
    pub mid: String,
    pub high: String,
    pub angle: f64,
    pub spread: f64,
}

pub fn analyze_track_data<F>(file_path: &str, mut progress_cb: F) -> Result<AnalysisResult, String>
where
    F: FnMut(f64, &str),
{
    let p = Path::new(file_path);
    if !p.exists() {
        return Err(format!("找不到文件: {}", file_path));
    }

    let file_title = p.file_name().and_then(|n| n.to_str()).unwrap_or("AUDIO TRACK").to_string();

    progress_cb(10.0, "PROBING METADATA & STREAM ARCHITECTURE...");

    let probe_output = create_cmd("ffprobe")
        .args([
            "-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", file_path,
        ])
        .output()
        .map_err(|e| format!("ffprobe 执行失败: {}", e))?;

    let probe_raw = String::from_utf8_lossy(&probe_output.stdout).to_string();
    let probe_json: serde_json::Value = serde_json::from_str(&probe_raw)
        .map_err(|e| format!("ffprobe JSON 解析失败: {}", e))?;

    let fmt = &probe_json["format"];
    let streams = probe_json["streams"].as_array();
    let audio_stream = streams.and_then(|arr| {
        arr.iter().find(|s| s["codec_type"].as_str() == Some("audio"))
    });

    let codec_name = audio_stream
        .and_then(|s| s["codec_name"].as_str())
        .or_else(|| fmt["format_name"].as_str())
        .unwrap_or("UNKNOWN")
        .to_uppercase();

    let sample_rate: u32 = audio_stream
        .and_then(|s| s["sample_rate"].as_str())
        .and_then(|v| v.parse().ok())
        .unwrap_or(44100);

    let duration: f64 = fmt["duration"]
        .as_str()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);

    let parse_bits = |s: &serde_json::Value, key: &str| -> Option<u32> {
        s[key]
            .as_u64()
            .map(|v| v as u32)
            .or_else(|| s[key].as_str().and_then(|v| v.parse().ok()))
    };

    let bits_val: u32 = audio_stream
        .and_then(|s| parse_bits(s, "bits_per_raw_sample").or_else(|| parse_bits(s, "bits_per_sample")))
        .filter(|&v| v > 0)
        .or_else(|| {
            let sfmt = audio_stream.and_then(|s| s["sample_fmt"].as_str()).unwrap_or("");
            let cname = audio_stream.and_then(|s| s["codec_name"].as_str()).unwrap_or("");
            if sfmt.starts_with("s16") || sfmt.starts_with("u16") || cname.contains("16") {
                Some(16)
            } else if sfmt.starts_with("s24") || sfmt.starts_with("u24") || cname.contains("24") {
                Some(24)
            } else if sfmt.starts_with("s32") || sfmt.starts_with("flt") || cname.contains("32") {
                Some(24)
            } else {
                None
            }
        })
        .unwrap_or(16);

    let file_size_mb = match p.metadata() {
        Ok(m) => m.len() as f64 / (1024.0 * 1024.0),
        Err(_) => 0.0,
    };

    let format_str = fmt["format_name"].as_str().unwrap_or("").to_lowercase();
    let is_dsd = codec_name.contains("DSD") || format_str.contains("dsf") || format_str.contains("dff");

    let (dsd_name, spec_desc) = if is_dsd {
        let bit_clock = sample_rate * 8;
        let mult = bit_clock / 44100;
        let mhz = bit_clock as f64 / 1_000_000.0;
        let name = format!("DSD{}", mult);
        let spec = format!("DSD{} ({:.4} MHZ) // 1-BIT", mult, mhz);
        (name, spec)
    } else {
        let spec = if sample_rate >= 1000 {
            format!("{:.1} KHZ / {}-BIT", sample_rate as f64 / 1000.0, bits_val)
        } else {
            format!("{} HZ / {}-BIT", sample_rate, bits_val)
        };
        (codec_name.clone(), spec)
    };

    // 2. EBU R128
    progress_cb(32.0, "SCANNING EBU R128 LRA & TRUE PEAK INTEGRAL...");
    let ebur_output = create_cmd("ffmpeg")
        .args([
            "-nostats", "-i", file_path, "-filter_complex", "ebur128=peak=true", "-f", "null", "-",
        ])
        .output();

    let mut int_lufs = -99.9;
    let mut lra = 0.0;
    let mut true_peak = -99.9;

    if let Ok(o) = ebur_output {
        let err_str = String::from_utf8_lossy(&o.stderr);
        for line in err_str.lines() {
            let l = line.trim();
            if l.starts_with("I:") && l.contains("LUFS") {
                if let Some(val_str) = l.split_whitespace().nth(1) {
                    if let Ok(v) = val_str.parse::<f64>() {
                        int_lufs = v;
                    }
                }
            } else if l.starts_with("LRA:") && l.contains("LU") {
                if let Some(val_str) = l.split_whitespace().nth(1) {
                    if let Ok(v) = val_str.parse::<f64>() {
                        lra = v;
                    }
                }
            } else if l.starts_with("Peak:") && l.contains("dBFS") {
                if let Some(val_str) = l.split_whitespace().nth(1) {
                    if let Ok(v) = val_str.parse::<f64>() {
                        true_peak = v;
                    }
                }
            }
        }
    }

    // 3. astats
    progress_cb(52.0, "EVALUATING RMS ENERGY, CREST FACTOR & DC...");
    let astats_output = create_cmd("ffmpeg")
        .args([
            "-nostats", "-i", file_path, "-af", "astats=measure_perchannel=1", "-f", "null", "-",
        ])
        .output();

    let mut overall_rms = -99.9;
    let mut overall_peak = -99.9;
    let mut dc_offset_l = 0.0;
    let mut dc_offset_r = 0.0;
    let mut bit_depth_active: u32 = 24;

    if let Ok(o) = astats_output {
        let err_str = String::from_utf8_lossy(&o.stderr);
        for line in err_str.lines() {
            let l = line.trim();
            if l.contains("Peak level dB:") && (overall_peak - (-99.9f64)).abs() < 1e-4 {
                if let Some(part) = l.split(':').last() {
                    if let Ok(v) = part.trim().parse::<f64>() {
                        overall_peak = v;
                    }
                }
            } else if l.contains("RMS level dB:") && (overall_rms - (-99.9f64)).abs() < 1e-4 {
                if let Some(part) = l.split(':').last() {
                    if let Ok(v) = part.trim().parse::<f64>() {
                        overall_rms = v;
                    }
                }
            } else if l.contains("Bit depth:") {
                if let Some(part) = l.split(':').last() {
                    let first = part.trim().split('/').next().unwrap_or("24");
                    if let Ok(b) = first.trim().parse::<u32>() {
                        bit_depth_active = b;
                    }
                }
            } else if l.contains("DC offset:") {
                if let Some(part) = l.split(':').last() {
                    if let Ok(v) = part.trim().parse::<f64>() {
                        if dc_offset_l == 0.0 {
                            dc_offset_l = v;
                        } else {
                            dc_offset_r = v;
                        }
                    }
                }
            }
        }
    }

    let crest_factor = if overall_rms > -90.0 {
        overall_peak - overall_rms
    } else {
        0.0
    };

    // 4. Decode 44100Hz float PCM for Spatial & TT DR
    progress_cb(68.0, "DECODING SPATIAL CORRELATION & MICRO-DYNAMICS...");
    let pcm_output = create_cmd("ffmpeg")
        .args([
            "-v", "quiet", "-i", file_path, "-f", "f32le", "-ac", "2", "-ar", "44100", "-",
        ])
        .output();

    let pcm_bytes = pcm_output.map(|o| o.stdout).unwrap_or_default();
    let s = bytes_to_f32_vec(&pcm_bytes);

    let mut sum_l2 = 0.0f64;
    let mut sum_r2 = 0.0f64;
    let mut sum_lr = 0.0f64;

    for chunk in s.chunks_exact(2) {
        let l = chunk[0] as f64;
        let r = chunk[1] as f64;
        sum_l2 += l * l;
        sum_r2 += r * r;
        sum_lr += l * r;
    }

    let (corr, side_to_mid_db, lr_balance_db) = if !s.is_empty() {
        let denom = (sum_l2 * sum_r2).sqrt();
        let c = if denom > 1e-12 { sum_lr / denom } else { 1.0 };
        let mid_e = (sum_l2 + sum_r2 + 2.0 * sum_lr) / 4.0;
        let side_e = (sum_l2 + sum_r2 - 2.0 * sum_lr) / 4.0;
        let sm = if mid_e > 1e-12 && side_e > 1e-12 {
            10.0 * (side_e / mid_e).log10()
        } else {
            -99.0
        };
        let bal = 10.0 * ((sum_l2 + 1e-12) / (sum_r2 + 1e-12)).log10();
        (c, sm, bal)
    } else {
        (1.0, -99.0, 0.0)
    };

    // 5. Band-split spatial & TT DR
    progress_cb(80.0, "COMPUTING TRI-BAND POLAR PHASE MATRIX & TT DR...");
    let (low_bal, low_c, _low_sm) = run_band_spatial(file_path, "lowpass=f=250");
    let (mid_bal, mid_c, _mid_sm) = run_band_spatial(file_path, "bandpass=f=1000:width_type=h:w=1500");
    let (high_bal, high_c, _high_sm) = run_band_spatial(file_path, "highpass=f=2500");

    let step_samples = 44100 * 3 * 2;
    let n_blocks = s.len() / step_samples;
    let mut rms_l_list = Vec::with_capacity(n_blocks);
    let mut rms_r_list = Vec::with_capacity(n_blocks);
    let mut peak_l_list = Vec::with_capacity(n_blocks);
    let mut peak_r_list = Vec::with_capacity(n_blocks);
    let mut blocks_info = Vec::with_capacity(n_blocks);

    for b in 0..n_blocks {
        let chunk = &s[b * step_samples..(b + 1) * step_samples];
        let mut sl = 0.0f64;
        let mut sr = 0.0f64;
        let mut pl = 0.0f64;
        let mut pr = 0.0f64;
        let m = chunk.len() / 2;

        for pair in chunk.chunks_exact(2) {
            let l = pair[0] as f64;
            let r = pair[1] as f64;
            sl += l * l;
            sr += r * r;
            pl = pl.max(l.abs());
            pr = pr.max(r.abs());
        }

        rms_l_list.push((2.0 * sl / m as f64).sqrt());
        rms_r_list.push((2.0 * sr / m as f64).sqrt());
        peak_l_list.push(pl);
        peak_r_list.push(pr);

        let avg_rms = ((sl + sr) / (2.0 * m as f64)).sqrt();
        if avg_rms > 1e-7 {
            blocks_info.push(((b * 3) as f64, 20.0 * avg_rms.log10()));
        }
    }

    let top_n = 1.max((n_blocks as f64 * 0.2) as usize);
    let calc_dr = |r_list: &[f64], p_list: &[f64]| -> f64 {
        if r_list.is_empty() {
            return 0.0;
        }
        let mut sorted_r = r_list.to_vec();
        sorted_r.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top_r = &sorted_r[..top_n.min(sorted_r.len())];
        let rms_val = (top_r.iter().map(|x| x * x).sum::<f64>() / top_r.len() as f64).sqrt();

        let mut sorted_p = p_list.to_vec();
        sorted_p.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let sec_p = if sorted_p.len() > 1 { sorted_p[1] } else { sorted_p[0] };

        if rms_val > 1e-12 {
            20.0 * (sec_p / rms_val).log10()
        } else {
            0.0
        }
    };

    let dr_score = (calc_dr(&rms_l_list, &peak_l_list) + calc_dr(&rms_r_list, &peak_r_list)) / 2.0;

    let (_loudest_3s_time, _loudest_3s_rms, _quietest_3s_time, _quietest_3s_rms, macro_span) = if !blocks_info.is_empty() {
        let mut sorted_loud = blocks_info.clone();
        sorted_loud.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let (l_time, l_rms) = sorted_loud[0];

        let dynamic_gate = if overall_rms > -90.0 {
            (overall_rms - 32.0).max(-75.0)
        } else {
            -65.0
        };

        let active: Vec<_> = blocks_info.iter().filter(|x| x.1 > dynamic_gate).cloned().collect();
        if !active.is_empty() {
            let mut sorted_quiet = active;
            sorted_quiet.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let (q_time, q_rms) = sorted_quiet[0];
            (l_time, l_rms, q_time, q_rms, l_rms - q_rms)
        } else {
            (l_time, l_rms, 0.0, l_rms, 0.0)
        }
    } else {
        (0.0, overall_rms, 0.0, overall_rms, 0.0)
    };

    // 6. Hi-Res Frequency Extension
    progress_cb(92.0, "SCANNING ULTRASONIC STEP & CUTOFF CLIFF...");
    let rms_10k_20k = run_hires_band(file_path, "bandpass=f=15000:width_type=h:w=10000");
    let mut cutoff_desc = String::new();
    let mut cutoff_tag = "VALID";
    let mut rms_20k_24k = -120.0;
    let mut rms_24k_48k = -120.0;

    if is_dsd {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        rms_24k_48k = run_hires_band(file_path, "bandpass=f=36000:width_type=h:w=24000");
        cutoff_desc = "50.0 KHZ".to_string();
        cutoff_tag = "SACD/DSD STD";
    } else if sample_rate >= 88200 {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        rms_24k_48k = run_hires_band(file_path, "bandpass=f=36000:width_type=h:w=24000");
        if rms_20k_24k > -110.0 && (rms_10k_20k - rms_20k_24k < 38.0) {
            cutoff_desc = format!("{:.1} KHZ", sample_rate as f64 / 2000.0);
            cutoff_tag = "NATIVE HI-RES";
        } else {
            cutoff_desc = "22.05 KHZ".to_string();
            cutoff_tag = "UPSAMPLED CLIFF";
        }
    } else if sample_rate >= 48000 {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        cutoff_desc = "24.0 KHZ".to_string();
        cutoff_tag = "STD MASTER";
    } else {
        cutoff_desc = "22.05 KHZ".to_string();
        cutoff_tag = "STD CD REDBOOK";
    }

    progress_cb(100.0, "ANALYSIS COMPLETE");

    // Scope polar projection math
    let angle = (std::f64::consts::FRAC_PI_4) - (lr_balance_db * 0.12).clamp(-0.5, 0.5);
    let spread = (1.0 - corr.clamp(-1.0, 1.0)) * 0.35 + 0.08;

    let bal_dir = if lr_balance_db > 0.2 {
        "LEFT"
    } else if lr_balance_db < -0.2 {
        "RIGHT"
    } else {
        "CENTER"
    };

    let lufs_pct = ((int_lufs + 60.0) / 50.0 * 100.0).clamp(5.0, 98.0);
    let tp_pct = ((true_peak + 20.0) / 22.0 * 100.0).clamp(5.0, 98.0);

    let active_bits_str = if is_dsd {
        "1-BIT (DSD PDM STREAM)".to_string()
    } else {
        format!("{}-BIT (PHYSICAL)", bit_depth_active)
    };

    Ok(AnalysisResult {
        title: file_title,
        path: file_path.to_string(),
        codec: dsd_name,
        spec: spec_desc,
        dur: format_time(duration),
        duration_sec: duration,
        size_mb: file_size_mb,
        status: "ANALYZED // READY".to_string(),

        lufs: format!("{:+.2} LUFS", int_lufs),
        lufs_pct,
        tp: format!("{:+.2} DBFS", true_peak),
        tp_pct,
        dr: format!("DR{}", dr_score.round() as i32),
        dr_num: dr_score,
        crest: format!("{:+.2} DB", crest_factor),
        lra: format!("{:.2} LU", lra),
        span: format!("{:.2} DB", macro_span),

        cutoff: cutoff_desc,
        cutoff_tag: cutoff_tag.to_string(),
        band10k: format!("{:+.2} DBFS", rms_10k_20k),
        band20k: format!("{:+.2} DBFS", rms_20k_24k),
        band24k: format!("{:+.2} DBFS", rms_24k_48k),

        dc: format!("L: {:+.6} / R: {:+.6}", dc_offset_l, dc_offset_r),
        bits: active_bits_str,

        corr: format!("{:+.4}", corr),
        sm: format!("{:+.2} DB", side_to_mid_db),
        bal: format!("{:+.2} DB ({})", lr_balance_db, bal_dir),
        low: format!("{:+.2} DB | {:+.4}", low_bal, low_c),
        mid: format!("{:+.2} DB | {:+.4}", mid_bal, mid_c),
        high: format!("{:+.2} DB | {:+.4}", high_bal, high_c),
        angle,
        spread,
    })
}
