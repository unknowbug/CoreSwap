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
N = len(nodes)

# 活跃分析：节点 i 的值被引用到 max_parent[i]（所有引用 i 的父节点的最大索引）
# 后序：i 求值后活跃，直到最后一个引用它的父节点求值完。
read_fields = {6: ('a1','a2'), 7: ('a1','a2'), 8: ('a1','a2'), 9: ('a1','a2'),
               10: ('a1',), 11: ('a1',), 12: ('a1',), 13: ('a1',), 14: ('a1',), 15: ('a1',), 16: ('a1',),
               17: ('a1','a2','a3'), 20: ('a1',), 21: ('a1',)}
max_parent = [-1] * N
for i, n in enumerate(nodes):
    t = n['type']
    if t in read_fields:
        for f in read_fields[t]:
            c = n[f]
            if c >= 0 and c < i:
                max_parent[c] = max(max_parent[c], i)

# 贪心槽位分配：按结束时间（max_parent）排序复用
# 每个节点 i 在 [i, max_parent[i]] 区间活跃（max_parent=-1 表示无人引用，求值后立即释放）
slots = [-1] * N   # node -> slot
free_by_end = []   # 堆（按 max_parent 排序）——简单用列表
peak = 0
for i in range(N):
    # 释放已结束的槽位：max_parent[j] < i 的 j 释放
    # （简单线性扫描，N 小无所谓）
    live_slots = set()
    for j in range(i):
        if slots[j] >= 0 and max_parent[j] >= i:
            live_slots.add(slots[j])
    # 找可用槽位
    used = live_slots
    s = 0
    while s in used:
        s += 1
    slots[i] = s
    peak = max(peak, len(used) + 1)

print(f'节点数: {N}, val 槽位峰值: {peak}')
# 更精确：每个时刻的活跃数
from collections import defaultdict
active_at = defaultdict(int)
for i in range(N):
    for j in range(i, max_parent[i] + 1 if max_parent[i] >= 0 else i + 1):
        active_at[j] += 1
print(f'最大同时活跃节点数: {max(active_at.values())}')
