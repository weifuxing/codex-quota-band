"""Generate the small LVGL 8 UI font from the SDK's OFL Noto Sans SC font.
Requires Pillow. Run: python tools/generate_font.py --sdk <SiFli SDK root>
"""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
parser=argparse.ArgumentParser()
parser.add_argument('--sdk',type=Path,required=True)
parser.add_argument('--size',type=int,default=22,choices=[18,22,60])
parser.add_argument('--digits',action='store_true')
args=parser.parse_args()
app=Path(__file__).resolve().parents[1]
font_dir=args.sdk/'external/lvgl_v9/tests/src/test_files/fonts/noto'
font=ImageFont.truetype(str(font_dir/'NotoSansSC-Regular.ttf'),args.size)
texts='Codex额度5小时周重置暂无数据已同步缓存离线等待电脑蓝牙配对码请在输入分钟天未知连接不可用时间更新启动中'
codes=sorted({ord(c) for c in '-0123456789%'}) if args.digits else sorted(set(range(32,127))|{ord(c) for c in texts})
bitmap=[]
dsc=['    {0},']
for code in codes:
    ch=chr(code);x0,y0,x1,y1=font.getbbox(ch,anchor='ls')
    width,height=x1-x0,y1-y0
    img=Image.new('L',(max(width,1),max(height,1)))
    ImageDraw.Draw(img).text((-x0,-y0),ch,font=font,fill=255,anchor='ls')
    values=[round(value*15/255) for value in img.getdata()] if width and height else []
    index=len(bitmap)
    for i in range(0,len(values),2):bitmap.append((values[i]<<4)|(values[i+1] if i+1<len(values) else 0))
    dsc.append(f'    {{.bitmap_index={index}, .adv_w={round(font.getlength(ch)*16)}, .box_w={width}, .box_h={height}, .ofs_x={x0}, .ofs_y={-y1}}},')
rows=['    '+','.join(f'0x{x:02x}' for x in bitmap[i:i+20])+',' for i in range(0,len(bitmap),20)]
ascent,descent=font.getmetrics()
text='// Generated from Noto Sans SC (SIL Open Font License); see fonts/OFL.txt.\n#include "lvgl.h"\n'
text+='static const uint8_t bitmap[]={\n'+'\n'.join(rows)+'\n};\n'
text+='static const lv_font_fmt_txt_glyph_dsc_t glyphs[]={\n'+'\n'.join(dsc)+'\n};\n'
text+='static const uint16_t unicode_list[]={'+','.join(str(c-codes[0]) for c in codes)+'};\n'
text+=f'static const lv_font_fmt_txt_cmap_t cmaps[] = {{{{ .range_start={codes[0]}, .range_length={codes[-1]-codes[0]+1}, .glyph_id_start=1, .unicode_list=unicode_list, .list_length={len(codes)}, .type=LV_FONT_FMT_TXT_CMAP_SPARSE_TINY }}}};\n'
text+='static lv_font_fmt_txt_dsc_t dsc={ .glyph_bitmap=bitmap,.glyph_dsc=glyphs,.cmaps=cmaps,.cmap_num=1,.bpp=4,.bitmap_format=0 };\n'
text+=f'const lv_font_t cq_font_{args.size}={{ .get_glyph_dsc=lv_font_get_glyph_dsc_fmt_txt,.get_glyph_bitmap=lv_font_get_bitmap_fmt_txt,.line_height={ascent+descent},.base_line={descent},.dsc=&dsc }};\n'
(app/f'src/cq_font_{args.size}.c').write_text(text,encoding='utf-8')
(app/'fonts').mkdir(exist_ok=True)
(app/'fonts/OFL.txt').write_text((font_dir/'OFL.txt').read_text(encoding='utf-8'),encoding='utf-8')
print(f'Generated {len(codes)} glyphs, {len(bitmap)} bitmap bytes')
