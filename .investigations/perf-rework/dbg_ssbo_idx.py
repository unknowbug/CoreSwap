# 验证 _gen_spline 陈旧索引 bug：SSBO 节点索引 vs 解释器引用
import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings['noise_router']['final_density']
spec = importlib.util.spec_from_file_location('m', r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g.gen_shader(fd)

snodes = g.spline_ssbo_nodes
print(f'SSBO 样条节点数: {len(snodes)}')
for i, nd in enumerate(snodes):
    locs = g.spline_ssbo_locs[nd['locBegin']:nd['locBegin']+nd['n']]
    vk = g.spline_ssbo_val_kind[nd['valBegin']:nd['valBegin']+nd['n']]
    vf = g.spline_ssbo_val_f[nd['valBegin']:nd['valBegin']+nd['n']]
    vn = g.spline_ssbo_val_node[nd['valBegin']:nd['valBegin']+nd['n']]
    kinds = ['C' if k == 0 else 'S' for k in vk]
    refs = ['%d' % n if k == 1 else '%.3f' % v for k, n, v in zip(vk, vn, vf)]
    print(f'  SSBO[{i}] coordType={nd["coordType"]} n={nd["n"]} locs={[round(x,3) for x in locs]} vals={refs}')

# 解释器里 factor 子树：找 flat_cache(factor) 的 SPLINE 节点
nodes = g.df_nodes
names = {0:'CONST',1:'Y',2:'NOISE',3:'OLD',4:'SPLINE',5:'INTERP',6:'ADD',7:'MUL',8:'MIN',9:'MAX',10:'ABS',11:'SQ',12:'CUBE',13:'HNEG',14:'QNEG',15:'SQUEEZE',16:'CLAMP',17:'RANGE',18:'YCLAMP',19:'SNOISE',20:'BLEND',21:'FLAT'}
print('\n解释器 SPLINE 节点引用:')
for i, n in enumerate(nodes):
    if n['type'] == 4:
        print(f'  node[{i}] SPLINE a1={n["a1"]} (SSBO[{n["a1"]}])')
# factor 子树节点（node 33-37 附近）
print('\nfactor 子树节点:')
for i in range(28, 40):
    n = nodes[i]
    print(f'  node[{i}] {names.get(n["type"],"?")} a1={n["a1"]} a2={n["a2"]} a3={n["a3"]} f0={n["f0"]} f1={n["f1"]}')
