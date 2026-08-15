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
nodes = g.df_nodes
for i in (125, 124, 123, 3, 4, 5):
    n = nodes[i]
    print(f'node[{i}] type={n["type"]} a1={n["a1"]} a2={n["a2"]} a3={n["a3"]} f0={n["f0"]:.6f} f1={n["f1"]} f2={n["f2"]}')
# node 125 的递归展开
def show(i, depth=0):
    if i < 0 or depth > 3:
        return
    n = nodes[i]
    print(f'{"  "*depth}node[{i}] type={n["type"]} a1={n["a1"]} a2={n["a2"]} f0={n["f0"]:.6f} f1={n["f1"]} f2={n["f2"]}')
    for f in ('a1', 'a2'):
        if n[f] >= 0 and n[f] < i:
            show(n[f], depth + 1)
show(125)
