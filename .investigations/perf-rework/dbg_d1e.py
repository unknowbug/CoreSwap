import json, sys, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen
DFDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
NDIR = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json', encoding='utf-8'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(DFDIR, NDIR)
g.gen(fd)          # 旧路径（gen_cpu 用）
g.gen_shader(fd)   # _reset + gen_node
cy_bodies = [(idx, body) for idx, body in g.node_funcs if body and '(minY + (cy' in body]
print("body 含 cy 的 node_funcs:", len(cy_bodies))
for idx, body in cy_bodies[:3]:
    print(f"  df_{idx}: {body[:120]}")
# 这些 df_N 的 key 是什么？
print("node_funcs total:", len(g.node_funcs))
# interp samples 检查
if g.interp_funcs:
    s = g.interp_funcs[0][1][0]
    print("interp_0 sample[0]:", s[:100])
