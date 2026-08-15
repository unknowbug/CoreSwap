import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)
print("jagged keys:", sorted([k for k in g.normal_chain_index if 'jagged' in k]))
print("normal_chain_index total:", len(g.normal_chain_index))
print("coord_chains total:", len(g.coord_chains))
# 哪些 noise 有 c0 但没 c1？
allkeys = sorted(g.normal_chain_index.keys())
c0 = set(k[:-3] for k in allkeys if k.endswith('@c0'))
c1 = set(k[:-3] for k in allkeys if k.endswith('@c1'))
print("有 c0 无 c1 的 noise:", sorted(c0 - c1))
