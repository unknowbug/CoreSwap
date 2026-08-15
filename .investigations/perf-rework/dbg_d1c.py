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
g.gen_shader(fd)   # 内部 _reset + gen_node
idxs = [idx for idx, _ in g.node_funcs]
print("node_funcs count:", len(idxs), "unique:", len(set(idxs)))
dup = [i for i in set(idxs) if idxs.count(i) > 1]
print("duplicate idxs:", dup[:10])
# interp_funcs 数量（应该 5 不是 10）
print("interp_funcs:", len(g.interp_funcs))
