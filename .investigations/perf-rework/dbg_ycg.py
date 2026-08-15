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
# 所有 DF_Y_CLAMPED（18）节点
yc = [(i, n) for i, n in enumerate(g.df_nodes) if n['type'] == 18]
print(f'DF_Y_CLAMPED 节点: {len(yc)}')
for i, n in yc:
    print(f'  node[{i}]: from_y={n["f0"]} to_y={n["f1"]} from_v={n["f2"]} to_v={n["f3"]}')
# 谁引用它们（父节点）
print('引用者:')
for i, n in yc:
    refs = []
    for j, nn in enumerate(g.df_nodes):
        for f in ('a1', 'a2', 'a3'):
            if nn[f] == i:
                refs.append(j)
    print(f'  yc[{i}] 被引用: {refs} (类型 {[g.df_nodes[j]["type"] for j in refs]})')
# 顶层闭包里的 ycg
top = []
def closure_of(root):
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = g.df_nodes[i]
        visit(n['a1']); visit(n['a2']); visit(n['a3'])
    visit(root)
    return sorted(reach)
top_closure = closure_of(N - 1)
print(f'顶层闭包: {top_closure}')
print(f'顶层闭包内 yc: {[i for i in top_closure if g.df_nodes[i]["type"]==18]}')
