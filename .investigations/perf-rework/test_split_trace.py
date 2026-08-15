# test_split_trace.py —— 直接调 _gen_split_lines 复现 noodle_ridge_b 异常
import json, sys, os, traceback
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import dfc_gen
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
s = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = s['noise_router']['final_density']
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
g.gen_noise_manifest()
g.split_visited.clear()
try:
    lines = g._gen_split_lines(fd, "x", "y", "z")
    print(f'split lines: {len(lines)}')
    # 检查 noodle_ridge_b 是否生成
    nrb = [l for l in lines if 'noodle_ridge_b' in l or 'normals[19' in l]
    print(f'noodle_ridge_b 行: {len(nrb)}')
    for l in nrb[:3]:
        print(' ', l[:100])
except Exception:
    traceback.print_exc()
