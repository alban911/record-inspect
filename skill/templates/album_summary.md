---
title: "专辑母带质检看板：{album_name}"
album: "{album_name}"
album_path: "{album_path}"
track_count: {track_count}
mean_lufs: {mean_lufs}
max_true_peak: {max_true_peak}
album_dr_avg: "{album_dr_avg}"
outlier_count: {outlier_count}
created: "{created_date}"
tags:
  - album-qc
  - mastering-matrix
  - audio-inspection
---

# 💽 专辑母带一致性与全轨质检看板：{album_name}

> [!NOTE] 专辑综合概况
> - **曲目总数**：`{track_count}` 首 · 格式规格 `{spec_desc}`
> - **整专响度中枢**：`{mean_lufs} LUFS` · 最高 True Peak `{max_true_peak} dBFS`
> - **动态均值**：`{album_dr_avg}`
> - **质检异常/离群曲目**：`{outlier_summary}`

---

## 📋 一、全专声学物理指标对比矩阵

| 序号 | 曲目名称 (点击展开单轨详析) | 时长 | 综合响度 | LRA | True Peak | TT DR | 相关度 | 重心偏置 | 规格真实性 | 合规状态 |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
{matrix_rows}

---

## 🔍 二、专辑母带离群值与声学一致性分析

{consistency_analysis}

---

## 🚨 三、重点缺陷与高风险曲目预警

{risk_warnings}

---

## 💡 四、整专母带处理与听音重放建议

{album_advice}
