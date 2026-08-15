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

ns = len(g.spline_ssbo_nodes)
nl = len(g.spline_ssbo_locs)
nd = len(g.spline_ssbo_ders)
nvf = len(g.spline_ssbo_val_f)
nvk = len(g.spline_ssbo_val_kind)
nvn = len(g.spline_ssbo_val_node)
print(f'spline nodes={ns} locs={nl} ders={nd} val_f={nvf} val_kind={nvk} val_node={nvn}')

probs = []
for i, nd_ in enumerate(g.spline_ssbo_nodes):
    ct, n, lb, db, vb = nd_['coordType'], nd_['n'], nd_['locBegin'], nd_['derBegin'], nd_['valBegin']
    if lb + n > nl: probs.append((i, 'loc', lb, n, nl))
    if db + n > nd: probs.append((i, 'der', db, n, nd))
    if vb + n > nvk: probs.append((i, 'val', vb, n, nvk))
    if ct >= len(g.spline_coords): probs.append((i, 'coordType', ct))
    # nested spline 的 val_node 引用
    for j in range(n):
        k = g.spline_ssbo_val_kind[vb + j]
        if k == 1:
            node = g.spline_ssbo_val_node[vb + j]
            if node < 0 or node >= ns:
                probs.append((i, 'val_node', node, 'out of spline nodes'))
print(f'SSBO 越界: {len(probs)}')
for p in probs[:10]:
    print(' ', p)
if not probs:
    print('OK: spline SSBO 全部索引有效')
print('spline_coords 数:', len(g.spline_coords))
