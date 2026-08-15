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
s0 = g.interp_funcs[0][1]  # interp_0 samples
s5 = g.interp_funcs[5][1]  # interp_5 samples
print("interp_0 == interp_5 samples:", s0 == s5)
if s0 != s5:
    for i in range(8):
        print(f"  corner {i} identical:", s0[i] == s5[i])
        if s0[i] != s5[i]:
            print(f"    s0: {s0[i][:150]}")
            print(f"    s5: {s5[i][:150]}")
