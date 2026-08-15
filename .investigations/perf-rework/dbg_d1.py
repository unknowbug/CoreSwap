import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
# 只调 gen_node（不调旧 gen）
expr = g.gen_node(fd)
print("root expr:", expr[:100])
print("node_funcs:", len(g.node_funcs))
print("interp_funcs:", len(g.interp_funcs))
if g.interp_funcs:
    s0 = g.interp_funcs[0][1][0]
    print("interp_0 sample[0] head:", s0[:150])
    print("sample[0] has df_:", 'df_' in s0, "| has normal_noise(:", 'normal_noise(' in s0)
