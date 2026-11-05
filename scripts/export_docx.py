import docx
from docx.shared import Inches, Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.table import WD_TABLE_ALIGNMENT, WD_ALIGN_VERTICAL
from docx.oxml import parse_xml
from docx.oxml.ns import nsdecls, qn
import os

doc = docx.Document()

# 页面边距
sections = doc.sections
for s in sections:
    s.top_margin = Inches(0.8)
    s.bottom_margin = Inches(0.8)
    s.left_margin = Inches(0.9)
    s.right_margin = Inches(0.9)

# 基础样式
style_normal = doc.styles['Normal']
style_normal.font.name = 'Microsoft YaHei'
style_normal.font.size = Pt(10.5)
style_normal.font.color.rgb = RGBColor(0x33, 0x33, 0x33)
style_normal._element.rPr.rFonts.set(qn('w:eastAsia'), 'Microsoft YaHei')

def add_title(text):
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p.paragraph_format.space_before = Pt(12)
    p.paragraph_format.space_after = Pt(18)
    run = p.add_run(text)
    run.font.name = 'Microsoft YaHei'
    run.font.size = Pt(18)
    run.font.bold = True
    run.font.color.rgb = RGBColor(0x11, 0x18, 0x27)
    run._element.rPr.rFonts.set(qn('w:eastAsia'), 'Microsoft YaHei')

def add_heading_1(text):
    p = doc.add_paragraph()
    p.paragraph_format.space_before = Pt(14)
    p.paragraph_format.space_after = Pt(6)
    run = p.add_run(text)
    run.font.name = 'Microsoft YaHei'
    run.font.size = Pt(13)
    run.font.bold = True
    run.font.color.rgb = RGBColor(0x1F, 0x29, 0x37)
    run._element.rPr.rFonts.set(qn('w:eastAsia'), 'Microsoft YaHei')

def add_heading_2(text):
    p = doc.add_paragraph()
    p.paragraph_format.space_before = Pt(10)
    p.paragraph_format.space_after = Pt(4)
    run = p.add_run(text)
    run.font.name = 'Microsoft YaHei'
    run.font.size = Pt(11)
    run.font.bold = True
    run.font.color.rgb = RGBColor(0x37, 0x41, 0x51)
    run._element.rPr.rFonts.set(qn('w:eastAsia'), 'Microsoft YaHei')

def add_callout(text, title='提示'):
    table = doc.add_table(rows=1, cols=1)
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    table.autofit = False
    table.columns[0].width = Inches(6.5)
    cell = table.cell(0, 0)
    shading = parse_xml(r'<w:shd {} w:fill="F3F4F6"/>'.format(nsdecls('w')))
    cell._tc.get_or_add_tcPr().append(shading)
    borders = parse_xml(r'''
        <w:tcBorders {} >
            <w:top w:val="none"/>
            <w:left w:val="single" w:sz="24" w:space="0" w:color="3B82F6"/>
            <w:bottom w:val="none"/>
            <w:right w:val="none"/>
        </w:tcBorders>
    '''.format(nsdecls('w')))
    cell._tc.get_or_add_tcPr().append(borders)
    p = cell.paragraphs[0]
    p.paragraph_format.space_before = Pt(4)
    p.paragraph_format.space_after = Pt(4)
    r1 = p.add_run(f'【{title}】\n')
    r1.bold = True
    r1.font.color.rgb = RGBColor(0x1E, 0x40, 0xAF)
    r2 = p.add_run(text)
    r2.font.size = Pt(9.5)
    doc.add_paragraph().paragraph_format.space_after = Pt(4)

add_title('音频客观物理质检报告：1200')

summary_text = (
    '• 声明编制：中国民族音乐 · 女声独唱与民乐伴奏（主奏：古筝）\n'
    '• 规格格式：AAC · 44.1kHz / 24-bit Stereo · 时长 00:52 · 大小 0.83 MB\n'
    '• 动态指标：综合响度 -12.50 LUFS · 响度范围 (LRA) 5.50 LU · TT DR11\n'
    '• 峰值裕量：真峰值 (True Peak) +1.60 dBFS（严重超标）· 峰值因子 +16.78 dB\n'
    '• 声场相位：全频相关度 +0.7052 · 中侧比 -7.60 dB · 声道重心 +0.47 dB (偏右)\n'
    '• 工业合规结论：不合规（信号瞬态峰值击穿数字滤波上限，产生采样间削波失真）'
)
add_callout(summary_text, '核心指标速览')

add_heading_1('一、客观声学物理测量数据矩阵')

data = [
    ['物理检测维度', '测量参数', '实测数值', '行业参考门限标准', '物理数据对应分析'],
    ['响度与动态', '综合响度 (Integrated)', '-12.50 LUFS', '纯声乐/民乐 -18 ~ -22 LUFS', '电平经大幅度增益推升'],
    ['', '宏观响度范围 (LRA)', '5.50 LU', '原声编制 > 10 LU', '宏观动态范围窄，整体音量平坦连续'],
    ['', '全曲最高真峰值 (True Peak)', '+1.60 dBFS', '≤ -1.0 dBFS', '信号高脉冲峰值引发采样间硬削波'],
    ['', '瞬态峰值因子 (Crest Factor)', '+16.78 dB', '>14 dB 瞬态起音保留完好', '微观瞬态冲程明显，未被砖墙限制器彻底削平'],
    ['', 'TT DR 动态评分', 'DR11', 'DR10 ~ DR14', '微观瞬态动态结构良好'],
    ['', '宏观动态极差 (Macro Span)', '16.44 dB', '最响 3 秒与最弱 3 秒能量落差', '整体响度落差被约束在 16.44 dB 内'],
    ['立体声像与相位', '全频双声道相关性 (Correlation)', '+0.7052', '+0.40 ~ +0.80', '全局声场开阔度适中'],
    ['', '中/侧能量比 (Side/Mid Ratio)', '-7.60 dB', '-6.0 ~ -12.0 dB', '中央声像能量占优，侧向去相关能量适度'],
    ['', '声像重心偏置 (L/R Balance)', '+0.47 dB', '≤ ±0.30 dB', '右声道整体电平高出左声道 0.47 dB'],
    ['', '低频段 (<250Hz) 相位相关度', '+0.9267', '> +0.80', '低频能量高度集中于中轴，无低频发散'],
    ['', '中频段 (500Hz-2kHz) 相关度', '+0.4376', '独唱人声通常 > +0.75', '中频去相关能量高，虚拟中央声像发散'],
    ['', '高频段 (>2.5kHz) 相位相关度', '+0.7828', '> +0.20', '高频无负相关，无梳状滤波抵消'],
    ['频宽真实性', '实测有效截止频率 (Cutoff)', '22.05 kHz', '22.05 kHz (44.1k 上限)', '频宽完整，无低通截断'],
    ['', '10k-20k 高频能量', '-36.29 dBFS', '渐进阶梯衰减', '高频能量呈自然线性滚降'],
    ['电气纯净度', '直流偏置 (DC Offset)', '+0.000010', '0.000000', '零直流偏置漂移'],
    ['', '有效利用物理位深', '32-BIT', '浮点解码运算', '处理位深完整']
]

table = doc.add_table(rows=len(data), cols=5)
table.alignment = WD_TABLE_ALIGNMENT.CENTER
table.autofit = False

col_widths = [Inches(1.1), Inches(1.5), Inches(1.0), Inches(1.3), Inches(1.6)]
for i, row in enumerate(table.rows):
    trPr = row._tr.get_or_add_trPr()
    trPr.append(parse_xml(r'<w:cantSplit {}/>'.format(nsdecls('w'))))
    for j, cell in enumerate(row.cells):
        cell.width = col_widths[j]
        cell.vertical_alignment = WD_ALIGN_VERTICAL.CENTER
        cell.text = data[i][j]
        p = cell.paragraphs[0]
        p.paragraph_format.space_before = Pt(3)
        p.paragraph_format.space_after = Pt(3)
        p.paragraph_format.line_spacing = 1.05
        run = p.runs[0]
        run.font.name = 'Microsoft YaHei'
        run._element.rPr.rFonts.set(qn('w:eastAsia'), 'Microsoft YaHei')
        if i == 0:
            run.font.bold = True
            run.font.size = Pt(9)
            run.font.color.rgb = RGBColor(0xFF, 0xFF, 0xFF)
            shd = parse_xml(r'<w:shd {} w:fill="1F2937"/>'.format(nsdecls('w')))
            cell._tc.get_or_add_tcPr().append(shd)
        else:
            run.font.size = Pt(8.5)
            if i % 2 == 1:
                shd = parse_xml(r'<w:shd {} w:fill="F9FAFB"/>'.format(nsdecls('w')))
                cell._tc.get_or_add_tcPr().append(shd)
            if '+1.60 dBFS' in cell.text:
                run.font.bold = True
                run.font.color.rgb = RGBColor(0xDC, 0x26, 0x26)

doc.add_paragraph().paragraph_format.space_after = Pt(6)

add_heading_1('二、客观心理声学机制与重放性能解读')

add_heading_2('1. 声部声场拓扑与虚像聚焦')
p1 = doc.add_paragraph(
    '• 中频声像凝聚度发散（Phantom Center Spread）：中频区（500Hz - 2kHz）实测相关度仅为 +0.4376，中侧比为 -4.07 dB。在女声独唱声部占主导的频段，若声像居中，相关度通常在 +0.80 以上。实测较低的相关度与较高的侧向能量表明：混音中施加了较宽的立体声残响/扩散效果，或者伴奏录音在左右声道存在显著的时间差与相位差，削弱了中轴虚拟中央声像（Phantom Center Focus）的凝聚度。\n'
    '• 声道能量分布偏置（L/R Energy Asymmetry）：全频重心偏右 +0.47 dB。低频段（<250Hz）偏右 +0.98 dB，高频段（>2.5kHz）偏右 +0.30 dB，中频段（500Hz - 2kHz）偏左 -0.58 dB。数据客观呈现出不同频段声能在立体声场中的不对称倾斜。\n'
    '• 高频相位一致性：高频相关度为 +0.7828，全频无负相关，未出现梳状滤波反相抵消。'
)
p1.paragraph_format.line_spacing = 1.15
p1.paragraph_format.space_after = Pt(6)

add_heading_2('2. 动态性能与瞬态冲程')
p2 = doc.add_paragraph(
    '• 瞬态冲程保留（Transient Peak Retention）：实测瞬态峰值因子达 +16.78 dB，动态评分达 DR11。表明信号中短时尖峰脉冲（Peak）与有效值（RMS）之间维持着 16.78 dB 的显著跨度，起音上升沿（Attack）未被限制器完全截幅切平。\n'
    '• 宏观响度与声压跨度：EBU R128 LRA 为 5.50 LU，最响 3 秒（-13.17 dBFS）与最弱 3 秒（-29.61 dBFS）极差为 16.44 dB。宏观动态范围较窄，整体响度被压缩提升至连续稳定的电平水平。'
)
p2.paragraph_format.line_spacing = 1.15
p2.paragraph_format.space_after = Pt(6)

add_heading_1('三、现代母带工程与工业合规质检（EBU R128 / AES）')
p3 = doc.add_paragraph(
    '1. 信号瞬态击穿 True Peak（严重缺陷）：实测最高真峰值达到 +1.60 dBFS，严重突破 EBU R128（≤ -1.0 dBFS）防削波门限。机理为有限采样率下的窄脉冲在过采样数模转换（DAC）或有损转码重构时，样本间内插波形超出 0 dBFS，产生硬削波失真（Inter-sample Peaks Clipping）。\n'
    '2. 中频单声道兼容性：中频相关度 +0.4376 在进行单声道下混（L+R）时，会因侧向相位差引发梳状滤波效应，导致中频信号电平下降约 1.5 ~ 2.0 dB。\n'
    '3. 声道重心偏差：全频重心偏右 +0.47 dB，略超工业推荐的 ±0.30 dB 容限。'
)
p3.paragraph_format.line_spacing = 1.15
p3.paragraph_format.space_after = Pt(6)

add_heading_1('四、最终母带质检结论与修正方案')
advice_box = (
    '1. 防削波修正：总线电平整体下调 2.6 dB，启用具备过采样前瞻检测的 True Peak 限制器并将天花板设在 -1.0 dBFS，消除采样间削波失真；\n'
    '2. 中频声像校准：在中频段（500Hz-2.5kHz）通过 M/S 均衡器收敛侧向能量 1.5 ~ 2.0 dB，提升中轴虚像聚焦度；\n'
    '3. 声道平衡补偿：微调右声道衰减 0.4 dB，使左右声道能量趋向物理居中。'
)
add_callout(advice_box, '母带工程修正建议')

import sys
out_path = sys.argv[1] if len(sys.argv) > 1 else "audio_qc_report.docx"
doc.save(out_path)
print(f"SAVED DOCX TO: {out_path}")
