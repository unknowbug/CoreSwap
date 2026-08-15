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

# 打印 SSBO 前 40 个节点的 val_f 摘要（看顺序）
print('SSBO 节点顺序（前 40）：')
for i, nd in enumerate(g.spline_ssbo_nodes[:40]):
    n = nd['n']
    vf = g.spline_ssbo_val_f[nd['valBegin']:nd['valBegin']+n]
    vk = g.spline_ssbo_val_kind[nd['valBegin']:nd['valBegin']+n]
    print(f'  node[{i}] coordType={nd["coordType"]} n={n} val_kind={vk} val_f={[round(x,3) for x in vf][:4]}')

# node[33]（SPLINE 37）应该是什么 spline？——从 final_density 的树 trace
# 反查：node 33 的 spline_node=37 —— 检查 SSBO 37 与 33 的语义
print()
print(f'spline_ssbo_nodes[37] = {g.spline_ssbo_nodes[37]}')
print(f'  引用 node 37 的（父 spline val_node=37）: ', end='')
for i, nd in enumerate(g.spline_ssbo_nodes):
    for j in range(nd['n']):
        if g.spline_ssbo_val_kind[nd['valBegin']+j] == 1 and g.spline_ssbo_val_node[nd['valBegin']+j] == 37:
            print(f'node[{i}] (valBegin+j={nd["valBegin"]+j})')
            break
