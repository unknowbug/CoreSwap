# 隔离测试 _gen_spline：直接收集 factor 的 continents spline，观察嵌套引用录制
import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)

fac = json.load(open(dfdir + r'\overworld\factor.json', encoding='utf-8'))
# factor = flat_cache(cache_2d(add(10, mul(blend_alpha, add(-10, spline)))))
spline = fac['argument']['argument']['argument2']['argument2']['argument2'] if False else None
# 手动走: flat_cache -> cache_2d -> add -> mul -> add -> spline
x = fac
for k in ('argument', 'argument', 'argument2', 'argument2'):
    x = x[k]
# 上面走到了 mul(blend_alpha, add(-10, spline)) 的 add? 不对，手动拆：
sc = fac['argument']['argument']['argument2']['argument2']['argument2']['spline']
print('continents spline points:', [(p['location'], type(p['value']).__name__, (p['value'].get('coordinate') if isinstance(p['value'], dict) else p['value'])) for p in sc['points']])

call = g._gen_spline(sc)
print('collected call:', call)
print('SSBO nodes:', len(g.spline_ssbo_nodes))
for i, nd in enumerate(g.spline_ssbo_nodes):
    vk = g.spline_ssbo_val_kind[nd['valBegin']:nd['valBegin']+nd['n']]
    vf = g.spline_ssbo_val_f[nd['valBegin']:nd['valBegin']+nd['n']]
    vn = g.spline_ssbo_val_node[nd['valBegin']:nd['valBegin']+nd['n']]
    refs = ['S%d' % n if k == 1 else '%.3f' % v for k, n, v in zip(vk, vn, vf)]
    print(f'  SSBO[{i}] coordType={nd["coordType"]} n={nd["n"]} locs={[round(l,3) for l in g.spline_ssbo_locs[nd["locBegin"]:nd["locBegin"]+nd["n"]]]} vals={refs}')
print('cache keys:', list(g.spline_cache.keys())[:5], '... total', len(g.spline_cache))
