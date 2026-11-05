use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
extern "system" {
    fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    fn SetConsoleCP(wCodePageID: u32) -> i32;
}

fn set_windows_utf8() {
    #[cfg(windows)]
    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);
    }
}

fn format_time(seconds: f64) -> String {
    let total = seconds as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}

struct ProgressState {
    current_pct: f64,
    target_pct: f64,
    desc: String,
}

struct ProgressTracker {
    state: Arc<Mutex<ProgressState>>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    bar_len: usize,
}

impl ProgressTracker {
    fn new(bar_len: usize) -> Self {
        let state = Arc::new(Mutex::new(ProgressState {
            current_pct: 0.0,
            target_pct: 0.0,
            desc: "正在准备分析...".to_string(),
        }));
        let running = Arc::new(AtomicBool::new(true));

        let state_clone = Arc::clone(&state);
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                {
                    let mut s = state_clone.lock().unwrap();
                    if s.current_pct < s.target_pct {
                        let step = (s.target_pct - s.current_pct) * 0.15;
                        s.current_pct += step.max(0.4);
                        if s.current_pct > s.target_pct {
                            s.current_pct = s.target_pct;
                        }
                    }
                    let pct = s.current_pct as usize;
                    let filled = bar_len * pct / 100;
                    let empty = bar_len.saturating_sub(filled);
                    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
                    print!("\r  [{}] {:3}%  {:<35}", bar, pct, s.desc);
                    let _ = io::stdout().flush();
                }
                thread::sleep(Duration::from_millis(60));
            }
        });

        Self {
            state,
            running,
            handle: Some(handle),
            bar_len,
        }
    }

    fn set_stage(&self, target_pct: f64, desc: &str) {
        let mut s = self.state.lock().unwrap();
        s.target_pct = target_pct;
        s.desc = desc.to_string();
    }

    fn finish(&mut self, msg: &str) {
        {
            let mut s = self.state.lock().unwrap();
            s.target_pct = 100.0;
            s.current_pct = 100.0;
            s.desc = msg.to_string();
        }
        thread::sleep(Duration::from_millis(150));
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let bar = "█".repeat(self.bar_len);
        println!("\r  [{}] 100%  {:<35}", bar, msg);
        let _ = io::stdout().flush();
    }
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
    let output = Command::new("ffmpeg")
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
    let output = Command::new("ffmpeg")
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

fn analyze_track(file_path: &str) {
    let p = Path::new(file_path);
    if !p.exists() {
        println!("错误: 找不到指定文件: {}", file_path);
        return;
    }

    println!("\n[开始分析] 正在读取音频流与声学矩阵...");
    let mut progress = ProgressTracker::new(30);
    progress.set_stage(8.0, "正在提取音轨元数据与流信息...");

    // 1. ffprobe metadata
    let probe_output = Command::new("ffprobe")
        .args([
            "-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", file_path,
        ])
        .output();

    let probe_raw = match probe_output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => {
            progress.finish("元数据解析失败");
            println!(
                "\n解析音频格式元数据失败 (请确认系统已安装 ffmpeg 并加入环境变量 PATH)"
            );
            return;
        }
    };

    let probe_json: serde_json::Value = match serde_json::from_str(&probe_raw) {
        Ok(v) => v,
        Err(_) => {
            progress.finish("元数据 JSON 解析错误");
            return;
        }
    };

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

    let (dsd_name, _sample_rate_disp, _bits_val_disp) = if is_dsd {
        let bit_clock = sample_rate * 8;
        let mult = bit_clock / 44100;
        let mhz = bit_clock as f64 / 1_000_000.0;
        let name = format!("DSD{} ({:.4} MHz)", mult, mhz);
        (name, bit_clock, 1u32)
    } else {
        (codec_name.clone(), sample_rate, bits_val)
    };

    // 2. EBU R128
    progress.set_stage(30.0, "正在扫描 EBU R128 宏观响度与 True Peak...");
    let ebur_output = Command::new("ffmpeg")
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
    progress.set_stage(50.0, "正在统计全频 RMS、峰值与直流偏置...");
    let astats_output = Command::new("ffmpeg")
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
    progress.set_stage(65.0, "正在解调双声道声场与相关度矩阵...");
    let pcm_output = Command::new("ffmpeg")
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
    progress.set_stage(78.0, "正在计算分频段声部定位与 TT DR...");
    let (low_bal, low_c, low_sm) = run_band_spatial(file_path, "lowpass=f=250");
    let (mid_bal, mid_c, mid_sm) = run_band_spatial(file_path, "bandpass=f=1000:width_type=h:w=1500");
    let (high_bal, high_c, high_sm) = run_band_spatial(file_path, "highpass=f=2500");

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

    let (loudest_3s_time, loudest_3s_rms, quietest_3s_time, quietest_3s_rms, macro_span) = if !blocks_info.is_empty() {
        let mut sorted_loud = blocks_info.clone();
        sorted_loud.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let (l_time, l_rms) = sorted_loud[0];

        // 动态相对门限 + 绝对静音门限 (防磁带空带底噪误判，同时防古典极弱奏误杀)
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
    progress.set_stage(92.0, "正在扫描超声频段与真假 Hi-Res 断崖...");
    let rms_10k_20k = run_hires_band(file_path, "bandpass=f=15000:width_type=h:w=10000");
    let mut is_natural = true;
    let mut cutoff_desc = String::new();
    let mut rms_20k_24k = -120.0;
    let mut rms_24k_48k = -120.0;

    if is_dsd {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        rms_24k_48k = run_hires_band(file_path, "bandpass=f=36000:width_type=h:w=24000");
        cutoff_desc = "50.0 kHz (DSD模拟频宽上限)".to_string();
        is_natural = true;
    } else if sample_rate >= 88200 {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        rms_24k_48k = run_hires_band(file_path, "bandpass=f=36000:width_type=h:w=24000");
        if rms_20k_24k > -110.0 && (rms_10k_20k - rms_20k_24k < 38.0) {
            cutoff_desc = format!("{}.0 kHz", sample_rate / 2000);
            is_natural = true;
        } else {
            cutoff_desc = "约 22.05 kHz (存在截断断崖)".to_string();
            is_natural = false;
        }
    } else if sample_rate >= 48000 {
        rms_20k_24k = run_hires_band(file_path, "bandpass=f=22000:width_type=h:w=4000");
        cutoff_desc = "24.0 kHz".to_string();
    } else {
        cutoff_desc = "22.05 kHz".to_string();
    }

    progress.finish("分析完成，正在输出体检指标报告...");

    // Print Report
    let sep = "=".repeat(90);
    let subsep = "-".repeat(90);
    println!("\n{}", sep);
    println!("                    RECORD INSPECT - AUDIO QUALITY METRICS                      ");
    println!("{}", sep);
    println!("文件路径: {}", file_path);
    if is_dsd {
        println!(
            "格式规格: {} | 1-bit Stereo | 大小: {:.2} MB | 时长: {}",
            dsd_name, file_size_mb, format_time(duration)
        );
    } else {
        println!(
            "格式规格: {} | {} Hz | {}-bit Stereo | 大小: {:.2} MB | 时长: {}",
            codec_name, sample_rate, bits_val, file_size_mb, format_time(duration)
        );
    }
    println!("{}", subsep);

    println!("【1. 动态与响度指标 (Dynamics & Loudness)】");
    println!("  - 综合响度 (Integrated Loudness)    :  {:6.2} LUFS", int_lufs);
    println!("  - 宏观响度范围 (EBU R128 LRA)       :  {:6.2} LU", lra);
    println!("  - 全曲最高真峰值 (True Peak)        :  {:6.2} dBFS", true_peak);
    println!("  - 全曲平均有效值 (Overall RMS)      :  {:6.2} dBFS", overall_rms);
    println!("  - 瞬态峰值因子 (Crest Factor)       :  {:6.2} dB", crest_factor);
    let dr_round = dr_score.round() as i32;
    println!(
        "  - TT DR 动态评分 (TT Dynamic Range) :   DR{:<2} ({:5.2} dB)",
        dr_round, dr_score
    );
    println!(
        "  - 最响 3 秒有效值 (Loudest 3s RMS)  :  {:6.2} dBFS ({})",
        loudest_3s_rms,
        format_time(loudest_3s_time)
    );
    println!(
        "  - 最弱 3 秒有效值 (Quietest 3s RMS) :  {:6.2} dBFS ({})",
        quietest_3s_rms,
        format_time(quietest_3s_time)
    );
    println!("  - 宏观动态极差 (Macro Dynamic Span) :  {:6.2} dB", macro_span);
    println!();

    println!("【2. 空间声场与结像排布 (Spatial & Soundstage)】");
    println!("  - 全频双声道相关性 (Correlation)    :  {:+6.4}", corr);
    println!("  - 中/侧能量比 (Side/Mid Ratio)      :  {:6.2} dB", side_to_mid_db);
    println!("  - 全频左右声道重心 (L/R Balance)    :  {:+6.2} dB", lr_balance_db);
    println!(
        "  - 低频段 (<250Hz) 声像 / 相关度     :  {:+6.2} dB | Corr: {:+6.4} | S/M: {:6.2} dB",
        low_bal, low_c, low_sm
    );
    println!(
        "  - 中频段 (500Hz-2kHz) 声像 / 相关度 :  {:+6.2} dB | Corr: {:+6.4} | S/M: {:6.2} dB",
        mid_bal, mid_c, mid_sm
    );
    println!(
        "  - 高频段 (>2.5kHz) 声像 / 相关度    :  {:+6.2} dB | Corr: {:+6.4} | S/M: {:6.2} dB",
        high_bal, high_c, high_sm
    );
    println!();

    println!("【3. 频响延展与高规格真伪 (Frequency & Resolution)】");
    if is_dsd {
        println!("  - 标称采样率 / 标称位深             :  {} / 1-bit", dsd_name);
    } else {
        println!(
            "  - 标称采样率 / 标称位深             :  {} Hz / {}-bit",
            sample_rate, bits_val
        );
    }
    println!("  - 实测有效截止频率 (Cutoff Freq)    :  {}", cutoff_desc);
    println!(
        "  - 10kHz - 20kHz 频段能量            :  {:6.2} dBFS",
        rms_10k_20k
    );
    if is_dsd || sample_rate >= 88200 {
        println!(
            "  - 20kHz - 24kHz 超声频段能量        :  {:6.2} dBFS",
            rms_20k_24k
        );
        println!(
            "  - 24kHz - 48kHz 极限频段能量        :  {:6.2} dBFS",
            rms_24k_48k
        );
    }
    println!();

    println!("【4. 信号规范与本底纯净度 (Signal Hygiene)】");
    println!(
        "  - 直流偏移 (DC Offset)              :  L: {:+.6}, R: {:+.6}",
        dc_offset_l, dc_offset_r
    );
    if is_dsd {
        println!("  - 有效利用位深 (Active Bit Depth)   :  1-bit (音频带内等效 20~24-bit)");
    } else {
        println!(
            "  - 有效利用位深 (Active Bit Depth)   :  {}-bit",
            bit_depth_active
        );
    }
    println!("{}\n", sep);
}

fn clean_path(s: &str) -> String {
    s.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
}

fn main() {
    set_windows_utf8();

    let args: Vec<String> = env::args().collect();
    let mut interactive = false;
    let target_path = if args.len() > 1 {
        clean_path(&args[1])
    } else {
        interactive = true;
        let sep = "=".repeat(60);
        println!("\n{}", sep);
        println!("          RECORD INSPECT - 发烧音频客观质量检测工具 (Rust Native)          ");
        println!("{}", sep);
        println!("提示: 支持 FLAC, WAV, DSF, DFF, APE, ALAC, AIFF, MP3 等全部格式。");
        print!("\n请将音频文件拖拽至此窗口并按回车 (或输入路径): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let stdin = io::stdin();
        if stdin.read_line(&mut input).is_ok() {
            clean_path(&input)
        } else {
            String::new()
        }
    };

    if !target_path.is_empty() {
        analyze_track(&target_path);
    } else {
        println!("未指定有效文件路径。");
    }

    if interactive {
        println!("按回车键退出程序...");
        let mut exit_buf = String::new();
        let _ = io::stdin().read_line(&mut exit_buf);
    }
}
