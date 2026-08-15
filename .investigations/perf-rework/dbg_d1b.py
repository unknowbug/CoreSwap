import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen_node(fd)
print("normal_chain_index keys sample:", sorted(g.normal_chain_index.keys())[:10])
print("has jagged@c1:", 'minecraft:jagged@c1' in g.normal_chain_index)
print("all keys with jagged:", [k for k in g.normal_chain_index if 'jagged' in k][:5])
print("coord_chains count:", len(g.coord_chains), "normal_chain_index count:", len(g.normal_chain_index))
