---
title: "{title}"
album: "{album}"
track_name: "{track_name}"
genre: "{genre}"
ensemble_type: "{ensemble_type}"
blind_test: true
file_path: "{file_path}"
format: "{format}"
spec: "{spec}"
duration: "{duration}"
file_size_mb: {file_size_mb}
integrated_lufs: {integrated_lufs}
lra: {lra}
true_peak_dbfs: {true_peak_dbfs}
crest_factor_db: {crest_factor_db}
tt_dr: "{tt_dr}"
macro_span_db: {macro_span_db}
correlation: {correlation}
side_mid_ratio_db: {side_mid_ratio_db}
lr_balance_db: {lr_balance_db}
low_corr: {low_corr}
mid_corr: {mid_corr}
high_corr: {high_corr}
cutoff_khz: {cutoff_khz}
active_bits: {active_bits}
compliance_status: "{compliance_status}"
created: "{created_date}"
tags:
  - audio-qc
  - mastering
  - blind-test
  - acoustic-metrics
---

# 🎵 音频客观物理质检报告：{track_name}

> [!NOTE] 盲测与核心指标速览
> - **用户声明编制**：体裁 `{genre}` · 编制 `{ensemble_type}`（严格盲测，不读取文件先验标签）
> - **规格格式**：`{spec}` · 时长 `{duration}` · `{file_size_mb} MB`
> - **动态指标**：综合响度 `{integrated_lufs} LUFS` · LRA `{lra} LU` · `{tt_dr}`
> - **峰值裕量**：True Peak `{true_peak_dbfs} dBFS` · 峰值因子 `+{crest_factor_db} dB`
> - **立体声场**：全频相关度 `{correlation}` · S/M 比 `{side_mid_ratio_db} dB` · 重心 `{lr_balance_db} dB`
> - **频宽规格**：截止点 `{cutoff_khz} kHz` · 有效利用位深 `{active_bits}-bit`
> - **工业合规性**：`{compliance_badge}`

---

## 📊 一、客观声学物理测量数据矩阵

| 物理检测维度 | 测量参数 | 实测数值 | 行业参考门限标准 |
| :--- | :--- | :--- | :--- |
| **响度与动态** | 综合响度 (Integrated Loudness) | **`{integrated_lufs} LUFS`** | 串流 -14 ~ -16 LUFS / 古典 -18 ~ -24 LUFS |
| | 宏观响度范围 (EBU R128 LRA) | **`{lra} LU`** | >12 LU 为大动态 / >15 LU 为极高动态 |
| | 全曲最高真峰值 (True Peak) | **`{true_peak_dbfs} dBFS`** | $\le -1.0\text{ dBFS}$ (无削波失真) |
| | 瞬态峰值因子 (Crest Factor) | **`+{crest_factor_db} dB`** | >14 dB 瞬态起音健康保留 |
| | TT DR 动态评分 (Dynamic Range) | **`{tt_dr}`** | DR8~DR14 经典母带安全区间 |
| | 宏观动态极差 (Macro Span) | **`{macro_span_db} dB`** | 最响 3 秒与最弱 3 秒差值 |
| **立体声像与相位** | 全频双声道相关性 (Correlation) | **`{correlation}`** | +0.40 ~ +0.80 (防单声道抵消) |
| | 中/侧能量比 (Side/Mid Ratio) | **`{side_mid_ratio_db} dB`** | -4.0 ~ -8.0 dB (标准开阔度) |
| | 声像重心偏置 (L/R Balance) | **`{lr_balance_db} dB`** | $\le \pm 0.3\text{ dB}$ (电气对称) |
| | 低频段 (<250Hz) 相位相关度 | **`{low_corr}`** | > +0.80 (防低频发散) |
| | 中频段 (500Hz-2kHz) 相关度 | **`{mid_corr}`** | +0.40 ~ +0.70 (核心结像) |
| | 高频段 (>2.5kHz) 相位相关度 | **`{high_corr}`** | > +0.20 (严禁负相关反相) |
| **频宽真实性** | 实测有效截止频率 (Cutoff) | **`{cutoff_khz} kHz`** | 96k 对应 48kHz / 44.1k 对应 22.05kHz |
| | 10k-20k / 20k-24k / 24k-48k | `{band_ladder}` | 高频衰减梯级 (排查假升频与超声噪声) |
| **电气纯净度** | 直流偏置 (DC Offset) | **`{dc_offset}`** | 0.000000 (零偏置) |
| | 有效利用物理位深 | **`{active_bits}-BIT`** | 标称与物理激活位深对照 |

---

## 🎧 二、视角 A：客观心理声学机制与重放性能解读 (Psychoacoustics & Playback Analysis)

### 1. 声场拓扑与空间定位 (Spatial Topology & Imaging)
{perspective_a_spatial}

### 2. 动态性能与功放驱动负荷 (Dynamic Range & Amplification Demands)
{perspective_a_dynamic}

### 3. 频域能量分布与频谱倾角 (Spectral Distribution & Tonal Balance)
{perspective_a_tonal}

---

## ⚖️ 三、视角 B：现代母带工程与合规质检（EBU R128 / AES）

{perspective_b_content}

---

## 🏁 四、最终母带质检结论与归档评级

- **动态评级**：`{dynamic_rating}`
- **声场与相位**：`{spatial_rating}`
- **频宽真实性**：`{frequency_rating}`
- **综合发布建议**：`{summary_advice}`
