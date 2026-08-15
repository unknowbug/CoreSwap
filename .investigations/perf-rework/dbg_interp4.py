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
names = {0: 'CONST', 1: 'Y', 2: 'NOISE', 3: 'OLD', 4: 'SPLINE', 5: 'INTERP',
         6: 'ADD', 7: 'MUL', 8: 'MIN', 9: 'MAX', 10: 'ABS', 11: 'SQ', 12: 'CUBE',
         13: 'HNEG', 14: 'QNEG', 15: 'SQUEEZE', 16: 'CLAMP', 17: 'RANGE',
         18: 'YCLAMP', 19: 'SNOISE', 20: 'BLEND', 21: 'FLAT'}

def closure_of(root):
    reach = set()
    def visit(i):
        if i < 0 or i >= N or i in reach: return
        reach.add(i)
        n = g.df_nodes[i]
        visit(n['a1']); visit(n['a2']); visit(n['a3'])
    visit(root)
    return sorted(reach)

root4 = g.interp_roots[4]
print(f'interp_4 delegate_root = {root4}')
c4 = closure_of(root4)
print(f'interp_4 闭包: {c4}')
for i in c4:
    n = g.df_nodes[i]
    t = n['type']
    info = f"type={t}({names.get(t,'?')}) a1={n['a1']} a2={n['a2']} a3={n['a3']}"
    if t in (2, 19):
        # 噪声 slot → 实例
        slot = n['a1']
        if slot < len(g.noise_slots):
            s = g.noise_slots[slot]
            info += f" slot->key={s['key']}"
    if t == 4:
        info += f" spline_node={n['a1']} aligned={n['a2']}"
    print(f'  node[{i}] {info}')
print('interp_4 root 值域: 在闭包内吗', root4 in c4)
