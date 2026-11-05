# Record Inspect

[![Release](https://github.com/alban911/record-inspect/actions/workflows/release.yml/badge.svg)](https://github.com/alban911/record-inspect/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows_x64-lightgrey.svg)]()

> **高精度母带声学测量、品质检视与 AI 盲测评估套件**
>
> 专为音频工程师、发烧友与母带质检设计的本地原生工具链：纯 Rust 声学计算内核 + 双风格工业仪表 GUI + 自动化批量 CLI + AI 智能体盲测评测 Skill。

[English](README.md) | [简体中文](README_zh.md)

---

## 核心特性

- **纯 Rust 高性能声学计算内核 (`src/engine.rs`)**
  - 单轨秒级全频段 DSP 分析，无重量级外部依赖。
  - 严格遵循工业级声学标准与 ITU-R / EBU 算法规范。
- **全维度物理指标测量**
  - **动态与响度**：EBU R128 综合响度 (`LUFS`)、宏观响度范围 (`LRA`)、4倍过采样真峰值 (`True Peak dBTP`)、瞬态峰值因子 (`Crest Factor`)、经典 `TT Dynamic Range (DR)`。
  - **声场与相位**：全频立体声相位相关系数 (`-1.0 ~ +1.0`)、侧边/中间信号能量比 (`Side/Mid Energy Ratio dB`)。
  - **假无损与截止频率识别**：高精度频谱边缘能量滚降检测，精准侦测 16kHz / 20kHz / 22.05kHz / 24kHz 等砖墙低通滤波（MP3 转码、Red Book CD 伪升频假 Hi-Res）。
  - **位深与直流偏置**：实际有效位深统计 (`Active Bit Depth`)、直流偏移 (`DC Offset`)。
- **双模工业级原生 GUI (`record-inspect-gui`)**
  - 基于 Tao + Wry (WebView2) 的轻量原生视窗，亚像素级响应。
  - **瑞士精密阳极氧化铝 (Swiss Anodized Aluminum)**：高对比度、清晰严谨的精密仪器风格。
  - **野兽派工业机架 (Brutalist Studio Rack)**：复古母带硬件棚风格，带真实物理质感的按键与指示灯。
  - 支持拖拽音频文件即刻测量与多标签数据呈现。
- **无头命令行工具 (`record-inspect-cli`)**
  - 支持单文件检视与文件夹递归批量分析，直接输出标准 JSON 或格式化控制台报告。
- **AI Agent 盲测评测规范 (`skill/`)**
  - 专为大型语言模型（如 Claude、GPT-4、Gemini）设计的标准化发烧母带盲测 Skill。
  - 强制规避曲目元数据偏见（曲名、艺术家、厂牌盲测），仅基于物理声学指标与用户输入的乐曲体裁、声学编制，输出客观、严肃的重放负荷与母带瑕疵诊断报告。

---

## 目录结构

```text
record-inspect/
├── .github/
│   └── workflows/
│       └── release.yml          # GitHub Actions 自动化多平台发布
├── assets/                      # 应用图标、矢量原图与 Windows 资源文件
│   ├── app.rc
│   ├── app_icon.ico
│   └── ...
├── docs/
│   └── METRICS_EXPLANATION.md   # 声学物理指标与工业判准详细手册
├── scripts/                     # Python 辅助脚本 (批处理、Docx 导出)
│   ├── batch_analyzer.py
│   ├── export_docx.py
│   └── record_inspect.py
├── skill/                       # AI Agent 盲测 Skill 规范与模板
│   ├── templates/
│   │   ├── album_summary.md
│   │   └── single_track.md
│   └── SKILL.md
├── src/                         # Rust 源码
│   ├── engine.rs                # 核心 DSP 测量引擎与数据模型
│   ├── gui_main.rs              # 原生桌面客户端入口
│   ├── main.rs                  # 命令行客户端入口
│   └── ui.html                  # 双模 GUI 界面渲染模板
├── Cargo.toml
├── LICENSE
├── README.md
└── README_zh.md
```

---

## 快速上手

### 1. 下载预编译文件
前往 [Releases 页面](https://github.com/alban911/record-inspect/releases) 下载最新的 Windows 64 位单文件可执行程序：
- `record-inspect-gui.exe`：双击即开的原生桌面仪表界面。
- `record-inspect-cli.exe`：命令行批量分析工具。

### 2. 源码编译 (Rust)
确保本地已安装 Rust (1.75+)：

```bash
# 克隆仓库
git clone https://github.com/alban911/record-inspect.git
cd record-inspect

# 编译高性能 Release 版本
cargo build --release

# 编译产物位于 target/release/
# - record-inspect-gui.exe
# - record-inspect-cli.exe
```

---

## 使用指南

### 原生 GUI 界面
直接启动 `record-inspect-gui.exe`，将需要质检的音频文件（WAV、FLAC、AIFF 等）拖入窗口区域，即可在毫秒内完成全曲物理声学校验：
- 顶部切换 **Swiss Aluminum** 或 **Brutalist Rack** 风格。
- 关键风险指标（如 True Peak 破零削波、假 Hi-Res 截止、动态压扁等）自动高亮警示。

### 命令行工具 (CLI)

```bash
# 单曲分析并输出易读格式
record-inspect-cli.exe "path/to/audio.flac"

# 输出机器可读的 JSON 格式
record-inspect-cli.exe --json "path/to/audio.flac"

# 递归分析整张专辑/目录
record-inspect-cli.exe "D:/Music/Master_Files"
```

### AI Agent Skill 盲测评估
本项目包含完整的 Agent Skill 规范（见 `skill/SKILL.md`）：
1. 复制 `skill/` 目录至 AI 助手的技能配置库（如 Antigravity、Claude Projects、Custom GPTs）。
2. 使用 `record-inspect-cli.exe --json` 提取曲目声学测量参数。
3. 提供物理数据及乐曲编制信息（例：*大编制交响乐*），AI 将遵循盲测协议输出专业声学诊断，禁止套用“通透”、“温暖”等虚浮主观形容词。

---

## 核心物理指标与判准

| 测量参数 | 标准/单位 | 工业安全/推荐区间 | 瑕疵/风险特征 |
| :--- | :--- | :--- | :--- |
| **Integrated Loudness** | EBU R128 (`LUFS`) | 古典: -24 ~ -18 / 流行: -14 ~ -9 | > -7 LUFS (动态严重受压) |
| **Loudness Range (LRA)**| EBU R128 (`LU`) | 宽动态交响: > 14 / 流行: 6 ~ 10 | < 4 LU (方波化超压缩) |
| **True Peak (TP)** | ITU-R BS.1770 (`dBTP`)| $\le -1.0\text{ dBTP}$ | $> 0.0\text{ dBTP}$ (DAC 重建削波) |
| **TT Dynamic Range** | `DR` 分级 | DR10 ~ DR16 (优秀自然动态) | $\le \text{DR6}$ (严重打压峰值) |
| **Crest Factor** | 瞬态比 (`dB`) | 12 ~ 18 dB | < 10 dB (起音瞬态磨损) |
| **Phase Correlation** | 相关系数 (`-1 ~ +1`) | +0.6 ~ +0.95 (稳定单声道兼容) | < 0 (存在反相相消风险) |
| **Cutoff Frequency** | 能量骤降点 (`kHz`) | 44.1k/48k 源: 无截断 / 96k+: > 40k | 20k/22.05k 陡降 (假高解析度) |

*详细声学原理与物理推导请参阅 [docs/METRICS_EXPLANATION.md](docs/METRICS_EXPLANATION.md)。*

---

## 许可证 (License)

本项目采用 [MIT 许可证](LICENSE)。
