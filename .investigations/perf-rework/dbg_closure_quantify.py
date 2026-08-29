# dbg_closure_quantify.py —— 量化 D25 闭包化消除的死计算：逐 interp / 顶层 的节点数与代价 op 数。
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
g.gen_df(fd)
layout = g._compute_val_layout()
closures = layout["closures"]
top_closure = layout["top_closure"]
nodes = g.df_nodes
n_nodes = len(nodes)

COSTY = {2, 3, 4, 19, 22}   # noise / old_blended / spline / shifted_noise / weird (昂贵叶子)
def typ(n):
    return nodes[n]["type"]
def summarize(name, idxs):
    N = len(idxs)
    costly = sum(1 for i in idxs if typ(i) in COSTY)
    return f"{name:10s} nodes={N:3d} costly={costly:3d}"

top_set = set(top_closure)
print("==== per-interp delegate closure (eval_df_base) ====")
total_nodes_before = 0
total_nodes_after = 0
total_cost_before = 0
total_cost_after = 0
for k, (closure, _, _, _) in enumerate(closures):
    full = set(range(n_nodes))
    csz = len(closure)
    nbf = len(full); naf = csz
    costbf = sum(1 for i in full if typ(i) in COSTY)
    costaf = sum(1 for i in closure if typ(i) in COSTY)
    total_nodes_before += nbf; total_nodes_after += naf
    total_cost_before += costbf; total_cost_after += costaf
    print(f"  interp {k}: closure={csz} (was {nbf})  costly={costaf} (was {costbf})")
print(f"  SUM eval_df_base: {total_nodes_before} -> {total_nodes_after} nodes, {total_cost_before} -> {total_cost_after} costly leaves")

print("\n==== top closure (eval_df per point) ====")
orphans = [i for i in range(n_nodes) if i not in top_set]
print(f"  TOP closure: {len(top_closure)} nodes (was {n_nodes}), costly={sum(1 for i in top_closure if typ(i) in COSTY)} (was {sum(1 for i in range(n_nodes) if typ(i) in COSTY)})")
print(f"  orphans eliminated per eval_df point: {len(orphans)} nodes, of which {sum(1 for i in orphans if typ(i) in COSTY)} are costly leaves")

print("\n==== workload ratio (per point, grid-cached production path) ====")
print(f"  eval_df node-dispatch: {n_nodes} -> {len(top_closure)}  ({len(top_closure)/n_nodes*100:.0f}% retained)")
print(f"  eval_df_base node-dispatch (grid build, per chunk): {total_nodes_before} -> {total_nodes_after}")
