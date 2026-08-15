import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)
N = len(g.df_nodes)
print(f'df_nodes = {N}')
# 顶层闭包（root = N-1）
def closure_of(root):
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = g.df_nodes[i]
        visit(n['a1']); visit(n['a2']); visit(n['a3'])
    visit(root)
    return sorted(reach)
top = closure_of(N - 1)
print(f'顶层闭包大小 = {len(top)}（root={N-1}）')
interp_nodes = [i for i, n in enumerate(g.df_nodes) if n['type'] == 5]
print(f'DF_INTERP 节点: {interp_nodes}')
for i in interp_nodes:
    print(f'  interp 节点 {i}: a1={g.df_nodes[i]["a1"]}（interp idx，不引 delegate）')
print(f'顶层闭包含 interp 节点: {[i for i in interp_nodes if i in top]}')
# 各 interp 闭包大小（对比）
for idx, root in enumerate(g.interp_roots):
    c = closure_of(root)
    print(f'interp_{idx} 闭包: {len(c)} (root={root})')
