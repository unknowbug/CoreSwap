# analyze_spline_table.py — 解析 CpuBackend spline 表，标注节点结构/深度/嵌套，为 MVP 强化建模提供基线
# 输入：mvp_spline_eval.cpp 内联表（NP[280] nodePack + SPLINE_VALNODE[245] + SPLINE_VALKIND[245]）
# 用法：python analyze_spline_table.py  >  out_analysis.txt
import sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# ---- 从 mvp_spline_eval.cpp 解析 NP / VALNODE / VALKIND ----
src = open(r"E:\PYTHON\CoreSwap\.investigations\perf-rework\vulkan-proto\mvp_spline_eval.cpp", encoding="utf-8").read()

def extract(name):
    m = re.search(r"static const int %s\[(\d+)\] = \{(.*?)\};" % name, src, re.S)
    assert m, f"cannot find {name}"
    body = re.sub(r"\s+", " ", m.group(2))
    return [int(x) for x in body.split(",") if x.strip() != ""]

NP     = extract("NP")
VALNODE= extract("SPLINE_VALNODE")
VALKIND= extract("SPLINE_VALKIND")
N = len(NP) // 5                # 节点数
V = len(VALNODE)                # value 条目数（245）
print(f"NP nodePack: nodes={N}, pk[280] len={len(NP)}")
print(f"SPLINE_VALNODE len={V}  SPLINE_VALKIND len={len(VALKIND)}")
print(f"SPLINE_NODES const = {N}")   # 应与 mvp 里 SPLINE_NODES=56 一致

# node[i] = {coordType, n, locBegin, derBegin, valBegin}
nodes = [(NP[i*5+0], NP[i*5+1], NP[i*5+2], NP[i*5+3], NP[i*5+4]) for i in range(N)]

# 反查：某个 node 的 valBegin..valBegin+n-1 是否为嵌套（VALKIND==1 → VALNODE 指向子 node）
import collections
child_edges = collections.defaultdict(list)   # node -> list[子 node]
for i,(ct,n,locB,derB,valB) in enumerate(nodes):
    for k in range(n):
        idx = valB + k
        if idx < V and VALKIND[idx] == 1:
            child_edges[i].append(VALNODE[idx])

# 计算每节点子树规模（递归，含环检测）
memo = {}
def subtree(id_, seen):
    if id_ in memo: return memo[id_]
    if id_ in seen:
        memo[id_] = 0  # 环：不计（防死循环）
        return 0
    seen = seen | {id_}
    n = 1
    for ch in child_edges.get(id_, []):
        n += subtree(ch, seen)
    memo[id_] = n
    return n

# 计算每节点子树深度
depth_memo = {}
def depth_of(id_, seen):
    if id_ in depth_memo: return depth_memo[id_]
    if id_ in seen: return 1
    seen = seen | {id_}
    chs = child_edges.get(id_, [])
    if not chs:
        depth_memo[id_] = 1; return 1
    d = 1 + max(depth_of(c, seen) for c in chs)
    depth_memo[id_] = d
    return d

print("\n=== 每节点结构（id: coordType n locBegin derBegin valBegin | 子节点) ===")
total_nested_edges = 0
for i,(ct,n,locB,derB,valB) in enumerate(nodes):
    chs = child_edges.get(i, [])
    if chs: total_nested_edges += 1
    depth = depth_of(i, frozenset())
    sz = subtree(i, frozenset())
    print(f"  node[{i:2d}] ct={ct} n={n} loc={locB} der={derB} val={valB}  depth={depth:2d} subSize={sz:2d}  child={chs}")

print(f"\n嵌套边总数（值为嵌套 spline 的边）: {total_nested_edges}")

# 每节点的 coordType 分布
print("\ncoordType 分布:", collections.Counter(nt[0] for nt in nodes))
print("n(点数) 分布:", collections.Counter(nt[1] for nt in nodes))

# 从根（未被任何节点引用的 node）看森林
all_refs = set()
for i, chs in child_edges.items():
    all_refs |= set(chs)
roots = [i for i in range(N) if i not in all_refs]
print(f"\n根节点（无人引用）: {roots}")
print(f"总引用节点数: {len(all_refs)}")
