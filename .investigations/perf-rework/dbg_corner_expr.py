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
g.gen_shader(fd)
s5 = g.interp_funcs[5][1]   # interp_5 samples
s0 = s5[0]
print("sample[0] length:", len(s0))
# 检查局部变量依赖
for var in ('cy','cx','cz','gx','gy','gz','chunkX','chunkZ','minY'):
    print(f"  contains '{var}':", var in s0)
print("sample[0] head:", s0[:180])
print("sample[0] tail:", s0[-180:])
