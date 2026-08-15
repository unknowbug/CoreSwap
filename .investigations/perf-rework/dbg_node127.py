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
for i in (127, 128, 126, 2):
    n = nodes[i]
    print(f'node[{i}] type={n["type"]} a1={n["a1"]} a2={n["a2"]} f0={n["f0"]:.6f} f1={n["f1"]}')
a1 = nodes[127]['a1']
print(f'node[127].a1 = {a1}')
if a1 >= 0:
    n = nodes[a1]
    print(f'  -> node[{a1}] type={n["type"]} a1={n["a1"]} a2={n["a2"]} f0={n["f0"]}')
    if n['a1'] >= 0:
        m2 = nodes[n['a1']]
        print(f'     -> node[{n["a1"]}] type={m2["type"]} f0={m2["f0"]}')
