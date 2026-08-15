import json, importlib.util, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]

spec = importlib.util.spec_from_file_location("m", r'E:\PYTHON\CoreSwap\.investigations\perf-rework\dfc_gen.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
g = m.DfcGen(dfdir, ndir)
g._reset_collect()
root = g.gen_df(fd)

nodes = g.df_nodes
print(f"df_nodes={len(nodes)} root={root}")

# 统计噪声节点（t==2 normal, t==19 shifted, t==3 old_blended）
from collections import Counter
tc = Counter(n["type"] for n in nodes)
print("type histogram (t:count):")
for t in sorted(tc): print(f"  {t}: {tc[t]}")

# 噪声实例的 key 含 @cN 的分布
ni = g.noise_instances
suf = Counter()
for _, p in ni:
    k = p["_key"]
    for c in range(8):
        if k.endswith(f"@c{c}"): suf[c] += 1; break
    else:
        suf['-'] += 1
print("noise instance suffix distribution:", dict(suf))

# 节点里引用噪声实例（t==2/19/3）的 A1 分布
noise_a1 = [n["a1"] for n in nodes if n["type"] in (2,3,19)]
print(f"noise node count={len(noise_a1)} (refs to noise instance idx)")

# 分析：8 角点 delegate 是否结构相同（只看 type+child 拓扑，忽略 a1 噪声实例索引）
def topo(n):
    t = n["type"]
    a1,a2,a3 = n["a1"],n["a2"],n["a3"]
    # 噪声/叶子节点：a1 是实例索引（角点相关）；结构节点：a1/a2/a3 是子节点索引
    if t in (2,3,19,0,1,18):  # 叶子（噪声/常量/y/y_clamped）——实例索引或常量
        return (t, n["f0"], n["f1"], n["f2"], n["f3"])
    return (t, a1, a2, a3)

# 找 interp 节点（t==5）
interps = [i for i,n in enumerate(nodes) if n["type"]==5]
print(f"interp nodes (t==5): {interps}")
for i in interps:
    print(f"  interp node {i}: a1={nodes[i]['a1']} (interp instance idx)")
