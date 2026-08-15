import json, importlib.util, sys, struct, math
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)

# 复用 dbg_full_sim 的函数（import）
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import importlib
import dbg_full_sim as S

# 直接测 interp_0（采样点 0 = (0,-64,0)）
r = S.interp_0(0, 0, -64, 0)
print(f'interp_0(0,-64,0) = {r}')
r = S.interp_0(128, 0, -62, 0)
print(f'interp_0(128,-62,0) = {r}')

# eval_df_base 顶层 root（157）的各关键节点
# 先看顶层闭包节点
N = len(S.nodes)
top_closure = S.g.interp_roots  # 不对
print(f'顶层 root = {N-1}')
# eval_df 顶层模拟（用 dbg_full_sim 的 eval_df）
r = S.eval_df(N-1, 0, 0, -64, 0)
print(f'eval_df(0,-64,0) = {r}')
r = S.eval_df(N-1, 128, 0, -62, 0)
print(f'eval_df(0,-62,0) = {r}')
