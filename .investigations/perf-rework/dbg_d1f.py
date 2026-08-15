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
g.gen_shader(fd)
b3 = [body for idx, body in g.node_funcs if idx == 3 and body][0]
print("df_3 body:", b3[:300])
print()
# 检查 y_clamped_gradient 分支是否真的在 _gen_leaf_expr
import inspect
src = inspect.getsource(dfc_gen.DfcGen._gen_leaf_expr)
print("_gen_leaf_expr 含 'y_clamped_gradient(iy':", 'y_clamped_gradient(iy' in src)
