import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen_shader(fd)
print("df_nodes:", len(g.df_nodes))
ob = [i for i,n in enumerate(g.df_nodes) if n['type'] == g.DF_OLD_BLENDED]
print("old_blended nodes:", len(ob), "idx:", ob[:10])
print("old_blended distinct a1:", sorted(set(g.df_nodes[i]['a1'] for i in ob)))
print("registry_defs:", len(g.registry_defs))
# registry body 是否含 df_ 引用
for fname, fexpr in g.registry_defs[:3]:
    print(f"  {fname}: {fexpr[:80]}")
