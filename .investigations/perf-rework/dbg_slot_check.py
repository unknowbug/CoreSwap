import json, importlib.util, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect(); g.gen_df(fd)
n = len(g.noise_instances)
print(f'实例数: {n}')
bad = []
for i, s in enumerate(g.noise_slots):
    mx = s['base'] + 7 * s['stride']
    if mx >= n:
        bad.append((i, s['key'], s['base'], s['stride'], mx))
print(f'slot 数: {len(g.noise_slots)}, 越界 slot: {bad if bad else "无"}')
# slot 的 kind 分布
kinds = {}
for s in g.noise_slots:
    kinds.setdefault((s['kind'], s['is_corner']), 0)
    kinds[(s['kind'], s['is_corner'])] += 1
print('slot kind 分布:', kinds)
for ct, expr in enumerate(g.spline_coords):
    print(f'spline_coord[{ct}] {expr[:110]}')
# eval_df 前向声明与定义签名
for i, n in enumerate(g.df_nodes[:10]):
    pass
print(f'df_nodes: {len(g.df_nodes)}')
