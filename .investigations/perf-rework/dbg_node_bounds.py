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
print(f'df_nodes: {N}')
# 各类型实际读取的子字段
read_fields = {
    6: ('a1', 'a2'), 7: ('a1', 'a2'), 8: ('a1', 'a2'), 9: ('a1', 'a2'),
    10: ('a1',), 11: ('a1',), 12: ('a1',), 13: ('a1',), 14: ('a1',), 15: ('a1',), 16: ('a1',),
    17: ('a1', 'a2', 'a3'), 20: ('a1',), 21: ('a1',),
}
problems = []
for i, n in enumerate(g.df_nodes):
    t = n['type']
    if t in read_fields:
        for field in read_fields[t]:
            v = n[field]
            if v < 0 or v >= N:
                problems.append((i, t, field, v, 'OOB'))
            elif v >= i:
                problems.append((i, t, field, v, 'forward-ref'))
print(f'真正越界/前向引用: {len(problems)}')
for p in problems[:15]:
    print(' ', p)
if not problems:
    print('OK: 所有被读子节点索引有效且后序')

# 统计每种类型节点数
from collections import Counter
tc = Counter(n['type'] for n in g.df_nodes)
print('类型直方图:', dict(sorted(tc.items())))
