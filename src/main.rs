use record_inspect_lib::analyze_track_data;
use std::env;
use std::io::{self, Write};
use std::path::Path;
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
                    print!("\r  [{}] {:3}%  {:<38}", bar, pct, s.desc);
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
        println!("\r  [{}] 100%  {:<38}", bar, msg);
        let _ = io::stdout().flush();
    }
}

fn analyze_track(file_path: &str) {
    let p = Path::new(file_path);
    if !p.exists() || !p.is_file() {
        println!("错误: 找不到指定音频文件: {}", file_path);
        return;
    }

    println!("\n[开始分析] 正在读取音频流与声学矩阵 (流式 DSP 内核)...");
    let mut progress = ProgressTracker::new(30);
    progress.set_stage(8.0, "正在启动底层声学解调引擎...");

    let res = analyze_track_data(file_path, |pct, desc| {
        let desc_zh = match desc {
            d if d.contains("PROBING") => "正在提取音轨元数据与流信息...",
            d if d.contains("EBU R128") => "正在扫描 EBU R128 宏观响度与 True Peak...",
            d if d.contains("RMS ENERGY") => "正在统计全频 RMS、峰值与直流偏置...",
            d if d.contains("SPATIAL CORRELATION") => "正在解调双声道声场与动态矩阵 (流式 DSP)...",
            d if d.contains("TRI-BAND") => "正在计算分频段声部定位与 TT DR...",
            d if d.contains("ULTRASONIC") => "正在扫描超声频段与真假 Hi-Res 断崖...",
            _ => desc,
        };
        progress.set_stage(pct, desc_zh);
    });

    match res {
        Ok(analysis) => {
            progress.finish("分析完成，正在输出体检指标报告...");
            analysis.print_terminal_report();
        }
        Err(e) => {
            progress.finish("分析中断或遇到错误");
            eprintln!("\n错误详情: {}", e);
        }
    }
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
