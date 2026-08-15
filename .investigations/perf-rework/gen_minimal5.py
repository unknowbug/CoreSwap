# minimal5 = minimal4 + 真实 spline_coord（slot 化查表）→ 定位 spline_coord 是否 TDR
import json, importlib.util, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)

comp = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal4.comp', encoding='utf-8').read()

# 从完整 comp 提取真实 spline_coord + NOISE_SLOT 表
comp_full = open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\final_density.comp', encoding='utf-8').read()
sc_start = comp_full.index('float spline_coord(')
sc_end = comp_full.index('int spline_find_range(')
spline_coord_real = comp_full[sc_start:sc_end].strip()

slot_start = comp_full.index('const int NOISE_SLOT_COUNT')
slot_end = comp_full.index('float eval_df_base(')
slot_tbl = comp_full[slot_start:slot_end].strip()

# 替换 minimal4 的简化 spline_coord
sc_simple_start = comp.index('float spline_coord(')
sc_simple_end = comp.index('int spline_find_range(')
comp = comp[:sc_simple_start] + spline_coord_real + '\n' + comp[sc_simple_end:]

# 插入 slot 表（在 spline 数据前）
comp = comp.replace('// ===== spline 数据驱动（完整）=====', slot_tbl + '\n// ===== spline 数据驱动（完整）=====')

open(r'E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\minimal5.comp', 'w', encoding='utf-8').write(comp)
print("minimal5.comp 生成")
