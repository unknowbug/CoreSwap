import json, sys, os, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)
g.gen_shader(fd)   # 先填 spline_coords
print("spline_coords:", g.spline_coords)
ssbo = g._spline_ssbo_glsl()
print("ssbo has normal_noise_0:", 'normal_noise_0' in ssbo)
print("spline funcs:", len(g.spline_funcs))
# spline_coord switch 里搜
for m in re.finditer(r'(?:normal_noise|interp_noise)_(\d+)', ssbo):
    pass
ids = set(int(m.group(1)) for m in re.finditer(r'(?:normal_noise|interp_noise)_(\d+)', ssbo))
print("noise ids in ssbo:", sorted(ids)[:10], "count:", len(ids))
