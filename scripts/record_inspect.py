import sys, os, subprocess, json, array, math, shutil, time, threading

# Windows UTF-8 console setup
if sys.platform == "win32":
    try:
        os.system("chcp 65001 >nul 2>&1")
        import io
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")
    except Exception:
        pass

def get_ffmpeg_path():
    p = shutil.which("ffmpeg")
    return p if p else "ffmpeg"

def get_ffprobe_path():
    p = shutil.which("ffprobe")
    return p if p else "ffprobe"

def format_time(seconds):
    m = int(seconds) // 60
    s = int(seconds) % 60
    return f"{m:02d}:{s:02d}"

class ProgressTracker:
    def __init__(self, bar_len=30):
        self.bar_len = bar_len
        self.running = True
        self.current_pct = 0.0
        self.target_pct = 0.0
        self.desc = "正在准备分析..."
        self.lock = threading.Lock()
        self.thread = threading.Thread(target=self._animate, daemon=True)
        self.thread.start()

    def set_stage(self, target_pct, desc):
        with self.lock:
            self.target_pct = float(target_pct)
            self.desc = desc

    def _animate(self):
        while self.running:
            with self.lock:
                if self.current_pct < self.target_pct:
                    step = (self.target_pct - self.current_pct) * 0.15
                    self.current_pct += max(0.4, step)
                    if self.current_pct > self.target_pct:
                        self.current_pct = self.target_pct
                pct = int(self.current_pct)
                filled = int(self.bar_len * pct / 100)
                bar = "█" * filled + "░" * (self.bar_len - filled)
                sys.stdout.write(f"\r  [{bar}] {pct:3d}%  {self.desc:<35}")
                sys.stdout.flush()
            time.sleep(0.06)

    def finish(self, msg="分析完成，正在输出体检指标报告..."):
        with self.lock:
            self.target_pct = 100.0
            self.current_pct = 100.0
            self.desc = msg
        time.sleep(0.15)
        self.running = False
        bar = "█" * self.bar_len
        sys.stdout.write(f"\r  [{bar}] 100%  {msg:<35}\n")
        sys.stdout.flush()

def analyze_track(file_path):
    if not os.path.exists(file_path):
        print(f"错误: 找不到指定文件: {file_path}")
        return

    ffmpeg = get_ffmpeg_path()
    ffprobe = get_ffprobe_path()

    print("\n[开始分析] 正在读取音频流与声学矩阵...")
    progress = ProgressTracker()
    progress.set_stage(8, "正在提取音轨元数据与流信息...")

    # 1. Probe metadata
    try:
        probe_cmd = [ffprobe, "-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", file_path]
        probe_raw = subprocess.check_output(probe_cmd, encoding="utf-8", errors="ignore")
        probe_data = json.loads(probe_raw)
    except Exception as e:
        progress.finish("元数据解析失败")
        print(f"\n解析音频格式元数据失败 (请确认系统已安装 ffmpeg 并加入环境变量 PATH): {e}")
        return

    fmt = probe_data.get("format", {})
    streams = probe_data.get("streams", [])
    audio_stream = next((s for s in streams if s.get("codec_type") == "audio"), {})

    codec_name = audio_stream.get("codec_name", fmt.get("format_name", "Unknown")).upper()
    sample_rate = int(audio_stream.get("sample_rate", 44100))
    channels = int(audio_stream.get("channels", 2))
    duration = float(fmt.get("duration", 0.0))
    bits_raw = audio_stream.get("bits_per_raw_sample") or audio_stream.get("bits_per_sample")
    if not bits_raw or str(bits_raw) == "0":
        sfmt = audio_stream.get("sample_fmt", "")
        cname = audio_stream.get("codec_name", "")
        if "s16" in sfmt or "u16" in sfmt or "16" in cname:
            bits_raw = 16
        elif "s24" in sfmt or "u24" in sfmt or "24" in cname:
            bits_raw = 24
        elif "s32" in sfmt or "flt" in sfmt or "32" in cname:
            bits_raw = 24
        else:
            bits_raw = 16
    bits_val = int(bits_raw)
    file_size_mb = os.path.getsize(file_path) / (1024 * 1024)

    format_str = fmt.get("format_name", "").lower()
    is_dsd = "dsd" in codec_name.lower() or "dsf" in format_str or "dff" in format_str
    if is_dsd:
        bit_clock = sample_rate * 8
        mult = bit_clock // 44100
        mhz = bit_clock / 1000000.0
        dsd_name = f"DSD{mult} ({mhz:.4f} MHz)"
    else:
        dsd_name = codec_name

    # 2. EBU R128 Loudness (Stage 2: 15% -> 35%)
    progress.set_stage(30, "正在扫描 EBU R128 宏观响度与 True Peak...")
    cmd_ebur = [ffmpeg, "-nostats", "-i", file_path, "-filter_complex", "ebur128=peak=true", "-f", "null", "-"]
    p_ebur = subprocess.run(cmd_ebur, capture_output=True, text=True, encoding="utf-8", errors="ignore")
    ebur_err = p_ebur.stderr
    
    int_lufs = -99.9
    lra = 0.0
    true_peak = -99.9

    for line in ebur_err.split("\n"):
        line_s = line.strip()
        if line_s.startswith("I:") and "LUFS" in line_s:
            try: int_lufs = float(line_s.split()[1])
            except: pass
        elif line_s.startswith("LRA:") and "LU" in line_s:
            try: lra = float(line_s.split()[1])
            except: pass
        elif line_s.startswith("Peak:") and "dBFS" in line_s:
            try: true_peak = float(line_s.split()[1])
            except: pass

    # 3. Overall astats (Stage 3: 35% -> 55%)
    progress.set_stage(50, "正在统计全频 RMS、峰值与直流偏置...")
    cmd_astats = [ffmpeg, "-nostats", "-i", file_path, "-af", "astats=measure_perchannel=1", "-f", "null", "-"]
    p_astats = subprocess.run(cmd_astats, capture_output=True, text=True, encoding="utf-8", errors="ignore")
    astats_err = p_astats.stderr

    overall_rms = -99.9
    overall_peak = -99.9
    dc_offset_l = 0.0
    dc_offset_r = 0.0
    bit_depth_active = 24

    for line in astats_err.split("\n"):
        l = line.strip()
        if "Peak level dB:" in l and overall_peak == -99.9:
            try: overall_peak = float(l.split(":")[-1].strip())
            except: pass
        elif "RMS level dB:" in l and overall_rms == -99.9:
            try: overall_rms = float(l.split(":")[-1].strip())
            except: pass
        elif "Bit depth:" in l:
            try:
                parts = l.split(":")[-1].strip().split("/")
                bit_depth_active = int(parts[0])
            except: pass
        elif "DC offset:" in l:
            try:
                val = float(l.split(":")[-1].strip())
                if dc_offset_l == 0.0: dc_offset_l = val
                else: dc_offset_r = val
            except: pass

    crest_factor = overall_peak - overall_rms if overall_rms > -90 else 0.0

    # 4. Decode 44100Hz float PCM for Spatial & TT DR (Stage 4: 55% -> 70%)
    progress.set_stage(65, "正在解调双声道声场与相关度矩阵...")
    cmd_pcm = [ffmpeg, "-v", "quiet", "-i", file_path, "-f", "f32le", "-ac", "2", "-ar", "44100", "-"]
    raw_pcm = subprocess.check_output(cmd_pcm)
    s = array.array("f")
    s.frombytes(raw_pcm)

    L = s[0::2]
    R = s[1::2]
    N = len(L)

    if N > 0:
        sum_L2 = sum(x*x for x in L)
        sum_R2 = sum(x*x for x in R)
        sum_LR = sum(L[i]*R[i] for i in range(N))

        denom = math.sqrt(sum_L2 * sum_R2)
        corr = sum_LR / denom if denom > 1e-12 else 1.0

        mid_e = (sum_L2 + sum_R2 + 2*sum_LR) / 4.0
        side_e = (sum_L2 + sum_R2 - 2*sum_LR) / 4.0
        side_to_mid_db = 10 * math.log10(side_e / mid_e) if mid_e > 1e-12 and side_e > 1e-12 else -99.0
        lr_balance_db = 10 * math.log10((sum_L2 + 1e-12) / (sum_R2 + 1e-12))
    else:
        corr = 1.0
        side_to_mid_db = -99.0
        lr_balance_db = 0.0

    # Band-split spatial & TT DR (Stage 5: 70% -> 85%)
    progress.set_stage(78, "正在计算分频段声部定位与 TT DR...")
    def get_band_spatial(b_filter):
        cmd = [ffmpeg, "-v", "quiet", "-i", file_path, "-af", b_filter, "-f", "f32le", "-ac", "2", "-ar", "44100", "-"]
        r = subprocess.check_output(cmd)
        sb = array.array("f")
        sb.frombytes(r)
        if len(sb) == 0: return 0.0, 1.0, -99.0
        Lb, Rb = sb[0::2], sb[1::2]
        sL2 = sum(x*x for x in Lb)
        sR2 = sum(x*x for x in Rb)
        sLR = sum(Lb[i]*Rb[i] for i in range(len(Lb)))
        d = math.sqrt(sL2 * sR2)
        c = sLR / d if d > 1e-12 else 1.0
        bal = 10 * math.log10((sL2 + 1e-12) / (sR2 + 1e-12))
        me = (sL2 + sR2 + 2*sLR) / 4.0
        se = (sL2 + sR2 - 2*sLR) / 4.0
        sm = 10 * math.log10(se / me) if me > 1e-12 and se > 1e-12 else -99.0
        return bal, c, sm

    low_bal, low_c, low_sm = get_band_spatial("lowpass=f=250")
    mid_bal, mid_c, mid_sm = get_band_spatial("bandpass=f=1000:width_type=h:w=1500")
    high_bal, high_c, high_sm = get_band_spatial("highpass=f=2500")

    step = 44100 * 3 * 2
    n_blocks = len(s) // step
    rms_l_list, rms_r_list = [], []
    peak_l_list, peak_r_list = [], []
    blocks_info = []

    for b in range(n_blocks):
        chunk = s[b*step : (b+1)*step]
        ch_l = chunk[0::2]
        ch_r = chunk[1::2]
        s_l = sum(x*x for x in ch_l)
        s_r = sum(x*x for x in ch_r)
        M = len(ch_l)
        rms_l_list.append(math.sqrt(2.0 * s_l / M))
        rms_r_list.append(math.sqrt(2.0 * s_r / M))
        peak_l_list.append(max(abs(x) for x in ch_l))
        peak_r_list.append(max(abs(x) for x in ch_r))
        avg_rms = math.sqrt((s_l + s_r) / (2.0 * M))
        if avg_rms > 1e-7:
            blocks_info.append((b * 3, 20 * math.log10(avg_rms)))

    top_n = max(1, int(n_blocks * 0.2))
    def calc_dr(r_list, p_list):
        if not r_list: return 0.0
        top_r = sorted(r_list, reverse=True)[:top_n]
        rms_val = math.sqrt(sum(x*x for x in top_r) / top_n)
        s_p = sorted(p_list, reverse=True)
        sec_p = s_p[1] if len(s_p) > 1 else s_p[0]
        return 20 * math.log10(sec_p / rms_val) if rms_val > 1e-12 else 0.0

    dr_score = (calc_dr(rms_l_list, peak_l_list) + calc_dr(rms_r_list, peak_r_list)) / 2.0

    if blocks_info:
        sorted_loud = sorted(blocks_info, key=lambda x: x[1], reverse=True)
        loudest_3s_time, loudest_3s_rms = sorted_loud[0]
        dynamic_gate = max(overall_rms - 32.0, -75.0) if overall_rms > -90.0 else -65.0
        active_blocks = [x for x in blocks_info if x[1] > dynamic_gate]
        if active_blocks:
            sorted_quiet = sorted(active_blocks, key=lambda x: x[1])
            quietest_3s_time, quietest_3s_rms = sorted_quiet[0]
            macro_span = loudest_3s_rms - quietest_3s_rms
        else:
            quietest_3s_time, quietest_3s_rms, macro_span = 0, loudest_3s_rms, 0.0
    else:
        loudest_3s_time, loudest_3s_rms = 0, overall_rms
        quietest_3s_time, quietest_3s_rms, macro_span = 0, overall_rms, 0.0

    # 5. Hi-Res Frequency Extension (Stage 6: 85% -> 98%)
    progress.set_stage(92, "正在扫描超声频段与真假 Hi-Res 断崖...")
    def get_hires_band(filter_str):
        cmd = [ffmpeg, "-v", "quiet", "-i", file_path, "-af", filter_str, "-f", "f32le", "-ac", "2", "-"]
        r = subprocess.check_output(cmd)
        sb = array.array("f")
        sb.frombytes(r)
        if len(sb) == 0: return -120.0
        val = math.sqrt(sum(x*x for x in sb) / len(sb))
        return 20 * math.log10(val + 1e-12)

    rms_10k_20k = get_hires_band("bandpass=f=15000:width_type=h:w=10000")
    is_natural = True
    if is_dsd:
        rms_20k_24k = get_hires_band("bandpass=f=22000:width_type=h:w=4000")
        rms_24k_48k = get_hires_band("bandpass=f=36000:width_type=h:w=24000")
        cutoff_desc = "50.0 kHz (DSD模拟频宽上限)"
        is_natural = True
    elif sample_rate >= 88200:
        rms_20k_24k = get_hires_band("bandpass=f=22000:width_type=h:w=4000")
        rms_24k_48k = get_hires_band("bandpass=f=36000:width_type=h:w=24000")
        if rms_20k_24k > -110.0 and (rms_10k_20k - rms_20k_24k < 38.0):
            cutoff_desc = f"{sample_rate // 2000}.0 kHz"
            is_natural = True
        else:
            cutoff_desc = "约 22.05 kHz (存在截断断崖)"
            is_natural = False
    elif sample_rate >= 48000:
        rms_20k_24k = get_hires_band("bandpass=f=22000:width_type=h:w=4000")
        rms_24k_48k = -120.0
        cutoff_desc = "24.0 kHz"
        is_natural = True
    else:
        rms_20k_24k = -120.0
        rms_24k_48k = -120.0
        cutoff_desc = "22.05 kHz"
        is_natural = True

    progress.finish("分析完成，正在输出体检指标报告...")

    # Print Report Table with 5-point ratings
    print("\n" + "=" * 90)
    print("                    RECORD INSPECT - AUDIO QUALITY METRICS                      ")
    print("=" * 90)
    print(f"文件路径: {file_path}")
    if is_dsd:
        print(f"格式规格: {dsd_name} | 1-bit Stereo | 大小: {file_size_mb:.2f} MB | 时长: {format_time(duration)}")
    else:
        print(f"格式规格: {codec_name} | {sample_rate:,} Hz | {bits_val}-bit Stereo | 大小: {file_size_mb:.2f} MB | 时长: {format_time(duration)}")
    print("-" * 90)
    print("【1. 动态与响度指标 (Dynamics & Loudness)】")
    print(f"  - 综合响度 (Integrated Loudness)    :  {int_lufs:6.2f} LUFS")
    print(f"  - 宏观响度范围 (EBU R128 LRA)       :  {lra:6.2f} LU")
    print(f"  - 全曲最高真峰值 (True Peak)        :  {true_peak:6.2f} dBFS")
    print(f"  - 全曲平均有效值 (Overall RMS)      :  {overall_rms:6.2f} dBFS")
    print(f"  - 瞬态峰值因子 (Crest Factor)       :  {crest_factor:6.2f} dB")
    print(f"  - TT DR 动态评分 (TT Dynamic Range) :   DR{round(dr_score):<2} ({dr_score:5.2f} dB)")
    print(f"  - 最响 3 秒有效值 (Loudest 3s RMS)  :  {loudest_3s_rms:6.2f} dBFS ({format_time(loudest_3s_time)})")
    print(f"  - 最弱 3 秒有效值 (Quietest 3s RMS) :  {quietest_3s_rms:6.2f} dBFS ({format_time(quietest_3s_time)})")
    print(f"  - 宏观动态极差 (Macro Dynamic Span) :  {macro_span:6.2f} dB")
    print()
    print("【2. 空间声场与结像排布 (Spatial & Soundstage)】")
    print(f"  - 全频双声道相关性 (Correlation)    :  {corr:+.4f}")
    print(f"  - 中/侧能量比 (Side/Mid Ratio)      :  {side_to_mid_db:6.2f} dB")
    print(f"  - 全频左右声道重心 (L/R Balance)    :  {lr_balance_db:+6.2f} dB")
    print(f"  - 低频段 (<250Hz) 声像 / 相关度     :  {low_bal:+6.2f} dB | Corr: {low_c:+.4f} | S/M: {low_sm:6.2f} dB")
    print(f"  - 中频段 (500Hz-2kHz) 声像 / 相关度 :  {mid_bal:+6.2f} dB | Corr: {mid_c:+.4f} | S/M: {mid_sm:6.2f} dB")
    print(f"  - 高频段 (>2.5kHz) 声像 / 相关度    :  {high_bal:+6.2f} dB | Corr: {high_c:+.4f} | S/M: {high_sm:6.2f} dB")
    print()
    print("【3. 频响延展与高规格真伪 (Frequency & Resolution)】")
    if is_dsd:
        print(f"  - 标称采样率 / 标称位深             :  {dsd_name} / 1-bit")
    else:
        print(f"  - 标称采样率 / 标称位深             :  {sample_rate:,} Hz / {bits_val}-bit")
    print(f"  - 实测有效截止频率 (Cutoff Freq)    :  {cutoff_desc}")
    print(f"  - 10kHz - 20kHz 频段能量            :  {rms_10k_20k:6.2f} dBFS")
    if is_dsd or sample_rate >= 88200:
        print(f"  - 20kHz - 24kHz 超声频段能量        :  {rms_20k_24k:6.2f} dBFS")
        print(f"  - 24kHz - 48kHz 极限频段能量        :  {rms_24k_48k:6.2f} dBFS")
    print()
    print("【4. 信号规范与本底纯净度 (Signal Hygiene)】")
    print(f"  - 直流偏移 (DC Offset)              :  L: {dc_offset_l:+.6f}, R: {dc_offset_r:+.6f}")
    if is_dsd:
        print(f"  - 有效利用位深 (Active Bit Depth)   :  1-bit (音频带内等效 20~24-bit)")
    else:
        print(f"  - 有效利用位深 (Active Bit Depth)   :  {bit_depth_active}-bit")
    print("=" * 90 + "\n")
    print("=" * 90 + "\n")

def main():
    interactive = False
    if len(sys.argv) > 1:
        target_path = sys.argv[1].strip("\"' ")
    else:
        interactive = True
        print("\n" + "=" * 60)
        print("          RECORD INSPECT - 发烧音频客观质量检测工具          ")
        print("=" * 60)
        print("提示: 支持 FLAC, WAV, DSF, DFF, APE, ALAC, AIFF, MP3 等全部格式。")
        try:
            target_path = input("\n请将音频文件拖拽至此窗口并按回车 (或输入路径): ").strip("\"' ")
        except (KeyboardInterrupt, EOFError):
            return

    if target_path:
        analyze_track(target_path)
    else:
        print("未指定有效文件路径。")

    if interactive:
        try:
            input("\n按回车键退出程序...")
        except:
            pass

if __name__ == "__main__":
    main()
