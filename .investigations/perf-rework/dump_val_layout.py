# dump_val_layout.py —— 打印 val 布局（bases/peaks/角点槽），供 gpu_corner_probe 精确定位
import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
layout = g._compute_val_layout()
print('per_sample =', layout['per_sample'])
print('val_slots_top =', layout['val_slots_top'])
print('top_peak =', layout['top_peak'])
print('bases =', layout['bases'])
for idx, (closure, pos, slot, peak) in enumerate(layout['closures']):
    b = layout['bases'][idx]
    print(f'interp_{idx}: base={b} peak={peak} len={len(closure)} root_pos={g.interp_root_pos[idx] if idx < len(g.interp_root_pos) else "?"}')
