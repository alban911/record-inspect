import math
import subprocess
from pathlib import Path
from PIL import Image

ASSETS_DIR = Path(r"E:\用户\alban911\Downloads\Software\record-inspect\rust-src\assets")
ARTIFACTS_DIR = Path(r"C:\Users\alban\.gemini\antigravity\brain\b35a3f74-e30f-47fa-aa66-de1a10b8ba72")
EDGE_PATH = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"

# =========================================================================
# 方案 1: 声学准星与黑胶母带 (Acoustic Reticle & Vinyl Master)
# =========================================================================
def get_svg_option1():
    # 生成同心音轨环
    grooves = []
    for r in range(85, 205, 10):
        opacity = 0.04 + (r % 20 == 0) * 0.05
        grooves.append(f'<circle cx="256" cy="256" r="{r}" fill="none" stroke="#ffffff" stroke-width="1" stroke-opacity="{opacity:.2f}"/>')
    
    # 准星刻度线
    ticks = []
    for angle_deg in range(0, 360, 15):
        rad = math.radians(angle_deg)
        r1 = 205
        r2 = 215 if angle_deg % 45 == 0 else (210 if angle_deg % 30 == 0 else 208)
        x1 = 256 + r1 * math.cos(rad)
        y1 = 256 + r1 * math.sin(rad)
        x2 = 256 + r2 * math.cos(rad)
        y2 = 256 + r2 * math.sin(rad)
        stroke = "#f59e0b" if angle_deg % 45 == 0 else "#64748b"
        width = "2" if angle_deg % 45 == 0 else "1"
        opacity = "0.8" if angle_deg % 45 == 0 else "0.4"
        ticks.append(f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{y2:.2f}" stroke="{stroke}" stroke-width="{width}" stroke-opacity="{opacity}"/>')

    grooves_str = "\n    ".join(grooves)
    ticks_str = "\n    ".join(ticks)

    # 45度倾斜的立体声利萨如曲线 (Lissajous)
    # x = 110 * sin(t), y = 65 * sin(2t + 0.35) 旋转 45 度
    points = []
    num_pts = 120
    for i in range(num_pts + 1):
        t = (2 * math.pi * i) / num_pts
        # 基础利萨如
        x0 = 100 * math.sin(t)
        y0 = 62 * math.sin(2 * t + 0.4)
        # 旋转 -45 度 (对齐立体声 L-R 轴)
        theta = -math.pi / 4
        xr = x0 * math.cos(theta) - y0 * math.sin(theta) + 256
        yr = x0 * math.sin(theta) + y0 * math.cos(theta) + 256
        points.append(f"{xr:.2f},{yr:.2f}")
    lissajous_path = "M " + " L ".join(points) + " Z"

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <!-- 背景渐变：深邃黑曜石与钛合金反光 -->
    <linearGradient id="bgGrad1" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#181c24"/>
      <stop offset="50%" stop-color="#0c0e12"/>
      <stop offset="100%" stop-color="#050608"/>
    </linearGradient>

    <!-- 黑胶唱片本体径向渐变 -->
    <radialGradient id="vinylGrad1" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#141820"/>
      <stop offset="40%" stop-color="#0d0f14"/>
      <stop offset="85%" stop-color="#080a0d"/>
      <stop offset="100%" stop-color="#161a22"/>
    </radialGradient>

    <!-- 唱片各向异性光影 (Vinyl Sheen) -->
    <linearGradient id="sheenGrad1" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.15"/>
      <stop offset="30%" stop-color="#ffffff" stop-opacity="0.0"/>
      <stop offset="70%" stop-color="#ffffff" stop-opacity="0.0"/>
      <stop offset="100%" stop-color="#ffffff" stop-opacity="0.15"/>
    </linearGradient>

    <!-- 中心利萨如声学光带渐变 -->
    <linearGradient id="lissaGrad" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#10b981"/>
      <stop offset="35%" stop-color="#fbbf24"/>
      <stop offset="70%" stop-color="#f59e0b"/>
      <stop offset="100%" stop-color="#ef4444"/>
    </linearGradient>

    <!-- 中心 CNC 金属圆盘渐变 -->
    <radialGradient id="cncGrad" cx="40%" cy="40%" r="60%">
      <stop offset="0%" stop-color="#475569"/>
      <stop offset="60%" stop-color="#1e293b"/>
      <stop offset="100%" stop-color="#0f172a"/>
    </radialGradient>

    <!-- 发光滤镜 -->
    <filter id="glow1" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="6" result="blur"/>
      <feComposite in="SourceGraphic" in2="blur" operator="over"/>
    </filter>
  </defs>

  <!-- 1. 外层平滑倒角容器 (Squircle Container) -->
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="url(#bgGrad1)"/>
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="none" stroke="#334155" stroke-width="2.5" stroke-opacity="0.8"/>
  <rect x="20" y="20" width="472" height="472" rx="100" ry="100" fill="none" stroke="#ffffff" stroke-width="1" stroke-opacity="0.08"/>

  <!-- 2. 四角工程定位标 (Precision Reticle Crop Marks) -->
  <path d="M 48 80 L 48 48 L 80 48" fill="none" stroke="#64748b" stroke-width="2.5" stroke-linecap="round"/>
  <path d="M 464 80 L 464 48 L 432 48" fill="none" stroke="#64748b" stroke-width="2.5" stroke-linecap="round"/>
  <path d="M 48 432 L 48 464 L 80 464" fill="none" stroke="#64748b" stroke-width="2.5" stroke-linecap="round"/>
  <path d="M 464 432 L 464 464 L 432 464" fill="none" stroke="#64748b" stroke-width="2.5" stroke-linecap="round"/>

  <!-- 3. 黑胶唱片基底 (Vinyl Disc Body) -->
  <circle cx="256" cy="256" r="216" fill="url(#vinylGrad1)" stroke="#1e293b" stroke-width="2"/>
  <circle cx="256" cy="256" r="216" fill="url(#sheenGrad1)" style="mix-blend-mode: overlay;"/>

  <!-- 4. 微观声槽物理同心圆 (Concentric Audio Grooves) -->
  {grooves_str}

  <!-- 5. 准星刻度盘环与刻度线 (Precision Reticle Scale) -->
  <circle cx="256" cy="256" r="206" fill="none" stroke="#f59e0b" stroke-width="1" stroke-dasharray="4,8" stroke-opacity="0.4"/>
  <circle cx="256" cy="256" r="82" fill="none" stroke="#64748b" stroke-width="1" stroke-opacity="0.5"/>
  {ticks_str}

  <!-- 6. 45度立体声 L/R 物理基准虚线与主十字线 -->
  <line x1="100" y1="412" x2="412" y2="100" stroke="#f59e0b" stroke-width="1.5" stroke-dasharray="3,6" stroke-opacity="0.5"/>
  <line x1="100" y1="100" x2="412" y2="412" stroke="#64748b" stroke-width="1.5" stroke-dasharray="3,6" stroke-opacity="0.3"/>
  <line x1="256" y1="44" x2="256" y2="468" stroke="#38bdf8" stroke-width="1" stroke-opacity="0.25"/>
  <line x1="44" y1="256" x2="468" y2="256" stroke="#38bdf8" stroke-width="1" stroke-opacity="0.25"/>

  <!-- L / R 通道标识 -->
  <text x="96" y="92" fill="#94a3b8" font-family="system-ui, sans-serif" font-size="14" font-weight="800" text-anchor="middle">L</text>
  <text x="416" y="92" fill="#f59e0b" font-family="system-ui, sans-serif" font-size="14" font-weight="800" text-anchor="middle">R</text>

  <!-- 7. 核心声学动态示波曲线 (Stereo Lissajous Loop) -->
  <path d="{lissajous_path}" fill="none" stroke="url(#lissaGrad)" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round" filter="url(#glow1)"/>
  <path d="{lissajous_path}" fill="none" stroke="#ffffff" stroke-width="1.5" stroke-opacity="0.8"/>

  <!-- 8. 中心 CNC 阳极氧化唱片轴心套环与红宝石激光测点 -->
  <circle cx="256" cy="256" r="36" fill="url(#cncGrad)" stroke="#64748b" stroke-width="2"/>
  <circle cx="256" cy="256" r="28" fill="none" stroke="#94a3b8" stroke-width="1" stroke-dasharray="2,3" stroke-opacity="0.6"/>
  <circle cx="256" cy="256" r="14" fill="#090d16" stroke="#475569" stroke-width="1.5"/>
  <circle cx="256" cy="256" r="5" fill="#10b981" filter="url(#glow1)"/>
  <circle cx="256" cy="256" r="2" fill="#ffffff"/>

  <!-- 9. 底部极客技术标印 -->
  <text x="256" y="478" fill="#64748b" font-family="monospace" font-size="11" font-weight="700" letter-spacing="4" text-anchor="middle">RECORD INSPECT // R128</text>
</svg>'''
    return svg


# =========================================================================
# 方案 2: 精密瑞士仪表 / 双环声学天平 (Swiss Precision Meter)
# =========================================================================
def get_svg_option2():
    # 生成表盘分贝刻度弧
    ticks = []
    # 顶部 240 度范围弧度从 -210 到 30 度
    for deg in range(-215, 36, 5):
        rad = math.radians(deg)
        r1 = 185
        is_major = (deg % 25 == 0)
        is_semi = (deg % 10 == 0)
        r2 = 202 if is_major else (195 if is_semi else 190)
        x1 = 256 + r1 * math.cos(rad)
        y1 = 256 + r1 * math.sin(rad)
        x2 = 256 + r2 * math.cos(rad)
        y2 = 256 + r2 * math.sin(rad)
        
        # 超过 0 dBFS (即最后 35 度) 用警示红，其余用精密白/金
        if deg > 5:
            stroke = "#ef4444"
        elif deg > -40:
            stroke = "#f59e0b"
        else:
            stroke = "#94a3b8"
        
        w = "2.5" if is_major else ("1.5" if is_semi else "1")
        ticks.append(f'<line x1="{x1:.2f}" y1="{y1:.2f}" x2="{x2:.2f}" y2="{x2:.2f}" stroke="{stroke}" stroke-width="{w}" stroke-linecap="round"/>')
    ticks_str = "\n    ".join(ticks)

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <!-- 阳极氧化铝拉丝外壳渐变 -->
    <linearGradient id="alumGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#334155"/>
      <stop offset="30%" stop-color="#1e293b"/>
      <stop offset="70%" stop-color="#0f172a"/>
      <stop offset="100%" stop-color="#020617"/>
    </linearGradient>

    <!-- 暖色表盘内衬渐变 (Swiss Nagra Amber Dial) -->
    <radialGradient id="dialGrad" cx="50%" cy="45%" r="60%">
      <stop offset="0%" stop-color="#1e1b18"/>
      <stop offset="70%" stop-color="#12100e"/>
      <stop offset="100%" stop-color="#080706"/>
    </radialGradient>

    <!-- 琥珀指针金色渐变 -->
    <linearGradient id="needleGrad" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#fbbf24"/>
      <stop offset="50%" stop-color="#f59e0b"/>
      <stop offset="100%" stop-color="#d97706"/>
    </linearGradient>

    <filter id="glow2" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="5" result="blur"/>
      <feComposite in="SourceGraphic" in2="blur" operator="over"/>
    </filter>
  </defs>

  <!-- 外壳底座 -->
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="url(#alumGrad)"/>
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="none" stroke="#64748b" stroke-width="2.5" stroke-opacity="0.8"/>
  <rect x="22" y="22" width="468" height="468" rx="98" ry="98" fill="none" stroke="#ffffff" stroke-width="1" stroke-opacity="0.1"/>

  <!-- 4角精密螺栓 (Hex Bolt Screws) -->
  <circle cx="56" cy="56" r="10" fill="#1e293b" stroke="#64748b" stroke-width="2"/>
  <circle cx="456" cy="56" r="10" fill="#1e293b" stroke="#64748b" stroke-width="2"/>
  <circle cx="56" cy="456" r="10" fill="#1e293b" stroke="#64748b" stroke-width="2"/>
  <circle cx="456" cy="456" r="10" fill="#1e293b" stroke="#64748b" stroke-width="2"/>

  <!-- 内部主表盘底圈 -->
  <circle cx="256" cy="256" r="210" fill="url(#dialGrad)" stroke="#334155" stroke-width="3"/>
  <circle cx="256" cy="256" r="208" fill="none" stroke="#f59e0b" stroke-width="1" stroke-opacity="0.3"/>

  <!-- 弧形刻度线簇 -->
  {ticks_str}

  <!-- 弧度标识文字 -->
  <text x="140" y="210" fill="#94a3b8" font-family="monospace" font-size="12" font-weight="700">-20</text>
  <text x="190" y="145" fill="#94a3b8" font-family="monospace" font-size="12" font-weight="700">-12</text>
  <text x="256" y="125" fill="#f59e0b" font-family="monospace" font-size="13" font-weight="800">-6</text>
  <text x="325" y="145" fill="#f59e0b" font-family="monospace" font-size="13" font-weight="800">0</text>
  <text x="370" y="210" fill="#ef4444" font-family="monospace" font-size="12" font-weight="800">+3</text>

  <!-- 示波声学环形光带 (中心立体声相关性与动态圈) -->
  <circle cx="256" cy="265" r="95" fill="none" stroke="#1e293b" stroke-width="14"/>
  <circle cx="256" cy="265" r="95" fill="none" stroke="#10b981" stroke-width="14" stroke-dasharray="160 400" stroke-linecap="round" filter="url(#glow2)"/>
  <circle cx="256" cy="265" r="95" fill="none" stroke="#f59e0b" stroke-width="14" stroke-dasharray="80 400" stroke-dashoffset="-180" stroke-linecap="round" filter="url(#glow2)"/>

  <!-- 阶梯波形量化柱 (Waveform Quantization Array) -->
  <g opacity="0.8">
    <rect x="180" y="295" width="6" height="24" rx="2" fill="#38bdf8"/>
    <rect x="192" y="285" width="6" height="34" rx="2" fill="#38bdf8"/>
    <rect x="204" y="270" width="6" height="49" rx="2" fill="#38bdf8"/>
    <rect x="216" y="255" width="6" height="64" rx="2" fill="#10b981"/>
    <rect x="228" y="240" width="6" height="79" rx="2" fill="#10b981"/>
    <rect x="240" y="230" width="6" height="89" rx="2" fill="#f59e0b"/>
    <rect x="252" y="220" width="6" height="99" rx="2" fill="#f59e0b"/>
    <rect x="264" y="235" width="6" height="84" rx="2" fill="#f59e0b"/>
    <rect x="276" y="250" width="6" height="69" rx="2" fill="#10b981"/>
    <rect x="288" y="265" width="6" height="54" rx="2" fill="#10b981"/>
    <rect x="300" y="280" width="6" height="39" rx="2" fill="#38bdf8"/>
    <rect x="312" y="290" width="6" height="29" rx="2" fill="#38bdf8"/>
    <rect x="324" y="300" width="6" height="19" rx="2" fill="#38bdf8"/>
  </g>

  <!-- 红色真峰值指示针 (Mastering True Peak Needle) -->
  <polygon points="253,350 259,350 257,110 255,110" fill="#ef4444" filter="url(#glow2)"/>
  <circle cx="256" cy="350" r="18" fill="#334155" stroke="#94a3b8" stroke-width="2"/>
  <circle cx="256" cy="350" r="7" fill="#0f172a"/>

  <!-- 仪表型号标识铭牌 -->
  <rect x="176" y="390" width="160" height="24" rx="6" fill="#0f172a" stroke="#334155" stroke-width="1.5"/>
  <text x="256" y="406" fill="#f59e0b" font-family="monospace" font-size="11" font-weight="800" letter-spacing="2" text-anchor="middle">PEAK // TRUE RMS</text>
</svg>'''
    return svg


# =========================================================================
# 方案 3: 极简唱针与动态波形切片 (Minimalist Stylus & Monogram)
# =========================================================================
def get_svg_option3():
    svg = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <linearGradient id="bgGrad3" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#111827"/>
      <stop offset="100%" stop-color="#030712"/>
    </linearGradient>

    <!-- 醒目战术高对比橙金 -->
    <linearGradient id="markGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#f59e0b"/>
      <stop offset="50%" stop-color="#fbbf24"/>
      <stop offset="100%" stop-color="#d97706"/>
    </linearGradient>

    <filter id="glow3" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="7" result="blur"/>
      <feComposite in="SourceGraphic" in2="blur" operator="over"/>
    </filter>
  </defs>

  <!-- 倒角底座 -->
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="url(#bgGrad3)"/>
  <rect x="16" y="16" width="480" height="480" rx="104" ry="104" fill="none" stroke="#374151" stroke-width="2.5"/>

  <!-- 外围极简四段圆弧 (象征唱片边缘与质检视界) -->
  <circle cx="256" cy="256" r="200" fill="none" stroke="#1f2937" stroke-width="6"/>
  <circle cx="256" cy="256" r="200" fill="none" stroke="#f59e0b" stroke-width="6" stroke-dasharray="140 180" stroke-linecap="round"/>
  <circle cx="256" cy="256" r="200" fill="none" stroke="#10b981" stroke-width="6" stroke-dasharray="90 230" stroke-dashoffset="-240" stroke-linecap="round"/>

  <!-- 几何抽象：大写 R / 唱针悬臂 与 脉冲声波的强力交错徽标 -->
  <!-- 左侧垂直声学主轴 (Main Acoustic Column) -->
  <rect x="120" y="110" width="42" height="292" rx="12" fill="#f3f4f6"/>

  <!-- 顶部精密唱臂悬梁向右延伸，形成 R 字顶环 / 示波波峰 -->
  <path d="M 162 110 L 300 110 C 365 110 405 150 405 210 C 405 270 365 305 300 305 L 162 305 Z" 
        fill="none" stroke="url(#markGrad)" stroke-width="42" stroke-linejoin="round" stroke-linecap="round" filter="url(#glow3)"/>
  <path d="M 162 110 L 300 110 C 365 110 405 150 405 210 C 405 270 365 305 300 305 L 162 305 Z" 
        fill="none" stroke="#ffffff" stroke-width="38" stroke-linejoin="round" stroke-linecap="round"/>

  <!-- R 字右斜下探针：斜向 45 度精密探针针尖 (Diamond Stylus Tip) -->
  <path d="M 270 290 L 375 402" stroke="url(#markGrad)" stroke-width="42" stroke-linecap="round"/>
  <!-- 探针尖端红宝石高光点 -->
  <circle cx="375" cy="402" r="12" fill="#ef4444" filter="url(#glow3)"/>
  <circle cx="375" cy="402" r="6" fill="#ffffff"/>

  <!-- 唱针尖端激发的微观音频脉冲波 (Micro Acoustic Waves) -->
  <path d="M 405 372 C 420 385 425 405 415 425" fill="none" stroke="#38bdf8" stroke-width="5" stroke-linecap="round" opacity="0.8"/>
  <path d="M 425 352 C 448 375 455 415 438 448" fill="none" stroke="#38bdf8" stroke-width="4" stroke-linecap="round" opacity="0.5"/>

  <!-- 内部音轨同心圆点标 -->
  <circle cx="260" cy="208" r="28" fill="#111827" stroke="#4b5563" stroke-width="8"/>
  <circle cx="260" cy="208" r="8" fill="#f59e0b"/>
</svg>'''
    return svg

def main():
    svgs = {
        "option1_reticle_master": get_svg_option1(),
        "option2_swiss_meter": get_svg_option2(),
        "option3_stylus_monogram": get_svg_option3(),
    }

    # 1. 保存 SVG 到 assets 和 artifacts
    for name, content in svgs.items():
        svg_file = ASSETS_DIR / f"{name}.svg"
        svg_file.write_text(content, encoding="utf-8")
        art_svg = ARTIFACTS_DIR / f"{name}.svg"
        art_svg.write_text(content, encoding="utf-8")
        print(f"Saved {svg_file.name}")

    # 2. 渲染为 512x512 PNG
    for name in svgs.keys():
        svg_path = (ASSETS_DIR / f"{name}.svg").resolve()
        png_path = (ASSETS_DIR / f"{name}.png").resolve()
        art_png = (ARTIFACTS_DIR / f"{name}.png").resolve()
        
        # 使用 Edge headless 截图渲染高质量 PNG
        cmd = [
            EDGE_PATH,
            "--headless",
            "--disable-gpu",
            f"--screenshot={str(png_path)}",
            "--window-size=512,512",
            "--default-background-color=00000000",
            f"file:///{str(svg_path).replace(chr(92), '/')}"
        ]
        subprocess.run(cmd, check=True)
        # 复制到 artifacts 目录
        art_png.write_bytes(png_path.read_bytes())
        print(f"Rendered PNG: {png_path.name}")

    # 3. 将方案 1 生成包含全套 Windows 尺寸的 record_inspect.ico
    # (16x16, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256)
    img_opt1 = Image.open(ASSETS_DIR / "option1_reticle_master.png")
    ico_path = ASSETS_DIR / "app_icon.ico"
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    img_opt1.save(ico_path, format="ICO", sizes=ico_sizes)
    print(f"Generated multi-res Windows ICO: {ico_path.name}")

if __name__ == "__main__":
    main()
