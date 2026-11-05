#![windows_subsystem = "windows"]

use tao::dpi::LogicalSize;
use record_inspect_lib::{analyze_track_data, AnalysisResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{http::Request, WebViewBuilder};

#[derive(Debug, Clone)]
enum AppEvent {
    Progress { pct: f64, desc: String },
    Complete(AnalysisResult),
    Error(String),
    Toast(String),
}

const UI_HTML: &str = include_str!("ui.html");

#[cfg(windows)]
extern "system" {
    fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut std::ffi::c_void;
    fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
    fn GlobalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

fn set_native_clipboard(text: &str) -> bool {
    #[cfg(windows)]
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return false;
        }
        EmptyClipboard();
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = utf16.len() * 2;
        let h_mem = GlobalAlloc(0x0002 /* GMEM_MOVEABLE */, size);
        let mut success = false;
        if !h_mem.is_null() {
            let ptr = GlobalLock(h_mem) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                GlobalUnlock(h_mem);
                if !SetClipboardData(13 /* CF_UNICODETEXT */, h_mem).is_null() {
                    success = true;
                } else {
                    GlobalFree(h_mem);
                }
            } else {
                GlobalFree(h_mem);
            }
        }
        CloseClipboard();
        success
    }
    #[cfg(not(windows))]
    false
}

#[cfg(windows)]
fn pick_audio_file() -> Option<String> {
    let script = r#"
[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null
$f = New-Object System.Windows.Forms.OpenFileDialog
$f.Filter = 'Audio Files (*.flac;*.wav;*.dsf;*.dff;*.ape;*.alac;*.aiff;*.mp3;*.m4a)|*.flac;*.wav;*.dsf;*.dff;*.ape;*.alac;*.aiff;*.mp3;*.m4a|All Files (*.*)|*.*'
$f.Title = 'Select Audio Track // 选择音频文件'
if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    [Console]::WriteLine($f.FileName)
}
"#;
    let mut cmd = std::process::Command::new("powershell");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]).output().ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

#[cfg(not(windows))]
fn pick_audio_file() -> Option<String> {
    None
}

fn trigger_analysis(
    file_path: String,
    busy_flag: Arc<Mutex<bool>>,
    proxy: tao::event_loop::EventLoopProxy<AppEvent>,
) {
    {
        let mut busy = busy_flag.lock().unwrap();
        if *busy {
            let _ = proxy.send_event(AppEvent::Toast("BUSY: ANALYSIS IN PROGRESS...".to_string()));
            return;
        }
        *busy = true;
    }

    let b_clone = Arc::clone(&busy_flag);
    thread::spawn(move || {
        let res = analyze_track_data(&file_path, |pct, desc| {
            let _ = proxy.send_event(AppEvent::Progress {
                pct,
                desc: desc.to_string(),
            });
        });

        match res {
            Ok(data) => {
                let _ = proxy.send_event(AppEvent::Complete(data));
            }
            Err(err) => {
                let _ = proxy.send_event(AppEvent::Error(err));
            }
        }

        let mut b = b_clone.lock().unwrap();
        *b = false;
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let icon_bytes = include_bytes!("../assets/icon_32.rgba");
    let window_icon = tao::window::Icon::from_rgba(icon_bytes.to_vec(), 32, 32).ok();

    let mut wb = WindowBuilder::new()
        .with_title("Record Inspect - 音频客观质量分析工具")
        .with_inner_size(LogicalSize::new(1240, 880))
        .with_min_inner_size(LogicalSize::new(980, 720));

    if let Some(icon) = window_icon {
        wb = wb.with_window_icon(Some(icon));
    }

    let window = wb.build(&event_loop)?;

    let busy_flag = Arc::new(Mutex::new(false));

    let drag_proxy = proxy.clone();
    let drag_busy = Arc::clone(&busy_flag);

    let ipc_proxy = proxy.clone();
    let ipc_busy = Arc::clone(&busy_flag);

    let handler = move |req: Request<String>| {
        let body = req.body().as_str();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
            match val["cmd"].as_str() {
                Some("copy") => {
                    if let Some(text) = val["text"].as_str() {
                        let ok = set_native_clipboard(text);
                        let msg = if ok {
                            val["msg"].as_str().unwrap_or("[ COPIED TO CLIPBOARD ]").to_string()
                        } else {
                            "[ ERR: FAILED TO ACCESS OS CLIPBOARD ]".to_string()
                        };
                        let _ = ipc_proxy.send_event(AppEvent::Toast(msg));
                    }
                }
                Some("open_file_dialog") => {
                    let p_clone = ipc_proxy.clone();
                    let b_clone = Arc::clone(&ipc_busy);
                    thread::spawn(move || {
                        if let Some(path) = pick_audio_file() {
                            trigger_analysis(path, b_clone, p_clone);
                        }
                    });
                }
                Some("analyze_path") => {
                    if let Some(path) = val["path"].as_str() {
                        let pb = PathBuf::from(path);
                        if pb.is_file() {
                            trigger_analysis(path.to_string(), Arc::clone(&ipc_busy), ipc_proxy.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    };

    let builder = WebViewBuilder::new()
        .with_html(UI_HTML)
        .with_ipc_handler(handler)
        .with_accept_first_mouse(true)
        .with_drag_drop_handler(move |e| {
            if let wry::DragDropEvent::Drop { paths, .. } = e {
                if let Some(first_path) = paths.first() {
                    let file_str = first_path.to_string_lossy().to_string();
                    trigger_analysis(file_str, Arc::clone(&drag_busy), drag_proxy.clone());
                }
            }
            true
        });

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let webview = builder.build(&window)?;

    // 若命令行传入了文件路径，自动启动分析
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let initial_file = args[1].trim_matches(|c| c == '"' || c == '\'').to_string();
        if PathBuf::from(&initial_file).exists() {
            trigger_analysis(initial_file, Arc::clone(&busy_flag), proxy.clone());
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(app_ev) => match app_ev {
                AppEvent::Progress { pct, desc } => {
                    let js = format!(
                        "if(window.__updateProgress){{window.__updateProgress({}, {:?});}}",
                        pct, desc
                    );
                    let _ = webview.evaluate_script(&js);
                }
                AppEvent::Complete(data) => {
                    if let Ok(json_str) = serde_json::to_string(&data) {
                        let js = format!(
                            "if(window.__updateTelemetry){{window.__updateTelemetry({});}}",
                            json_str
                        );
                        let _ = webview.evaluate_script(&js);
                    }
                }
                AppEvent::Error(err_msg) => {
                    let js = format!(
                        "if(window.__updateProgress){{window.__updateProgress(0, {:?});}}",
                        format!("ERR: {}", err_msg)
                    );
                    let _ = webview.evaluate_script(&js);
                }
                AppEvent::Toast(msg) => {
                    let js = format!(
                        "if(window.toastB){{window.toastB({:?});}}",
                        msg
                    );
                    let _ = webview.evaluate_script(&js);
                }
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}
