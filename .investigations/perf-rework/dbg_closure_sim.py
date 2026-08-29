# dbg_closure_sim.py —— 数值模拟证明 D25 闭包化求值 == 可达子树理想求值（逐位一致）。
# 用 mock 叶子（不依赖真实噪声）验证：① 闭包 = 可达子树 ② 闭包内 slot 求值顺序正确、无 liveness 复写破坏
# ③ 对每个 interp 顶层，闭包-slot 求值根值与「按节点索引的理想可达子树」求值根值逐位相等。
# 若任一面不等 → 闭包节点集 / SLOT 映射错误（会破坏对齐）。
import json, sys, os
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
read_fields = layout["read_fields"]
closures = layout["closures"]
top_closure = layout["top_closure"]
top_pos = layout["top_pos"]
top_slot = layout["top_slot"]
nodes = g.df_nodes
n_nodes = len(nodes)

# mock leaves (deterministic, distinct per (node_index, corner, a-slot)) — no real noise needed
def m_noise(slot, corner): return (float(slot) * 131.0 + float(corner)) * 0.001
def m_old(slot, corner):  return (float(slot) * 17.0 + float(corner)) * 0.5
def m_spline(a1, corner): return (float(a1) + float(corner)) * 2.0
def ws_scale(kind, v):
    if kind == 1:
        if v < -0.75: return 0.5
        if v < -0.5: return 0.75
        if v < 0.5: return 1.0
        return 2.0 if v < 0.75 else 3.0
    if v < -0.5: return 0.75
    if v < 0.0: return 1.0
    return 1.5 if v < 0.5 else 2.0

def eval_ref(subset_pos, root, corner, iy):
    """理想求值：按可达子树（节点索引升序 = 后序），val[node_index]。返回根值。"""
    val = [0.0] * n_nodes
    for ni in sorted(subset_pos.keys()):
        n = nodes[ni]; t = n["type"]
        if t == 0: r = n["f0"]
        elif t == 1: r = float(iy)
        elif t == 18:
            f0, f1, f2, f3 = n["f0"], n["f1"], n["f2"], n["f3"]
            r = 0.0 if f1 == f0 else f2 + max(0.0, min(1.0, (iy - f0) / (f1 - f0))) * (f3 - f2)
        elif t in (2, 19): r = m_noise(n["a1"], corner)
        elif t == 3: r = m_old(n["a1"], corner)
        elif t == 4: r = m_spline(n["a1"], corner)
        elif t == 22:
            v = val[n["a1"]]; r = ws_scale(int(n["f0"]), v) * abs(m_noise(n["a2"], corner))
        elif t in (20, 21): r = val[n["a1"]]
        elif t == 6: r = val[n["a1"]] + val[n["a2"]]
        elif t == 7: r = val[n["a1"]] * val[n["a2"]]
        elif t == 8: r = min(val[n["a1"]], val[n["a2"]])
        elif t == 9: r = max(val[n["a1"]], val[n["a2"]])
        elif t == 10: r = abs(val[n["a1"]])
        elif t == 11: v = val[n["a1"]]; r = v * v
        elif t == 12: v = val[n["a1"]]; r = v * v * v
        elif t == 13: v = val[n["a1"]]; r = v if v > 0 else v * 0.5
        elif t == 14: v = val[n["a1"]]; r = v if v > 0 else v * 0.25
        elif t == 15:
            v = val[n["a1"]]; c = 1.0 if v > 1 else (-1.0 if v < -1 else v); r = c / 2 - c * c * c / 24
        elif t == 16:
            v = val[n["a1"]]; r = n["f1"] if v > n["f1"] else (n["f0"] if v < n["f0"] else v)
        elif t == 17:
            inp = val[n["a1"]]; r = val[n["a2"]] if (n["f0"] <= inp < n["f1"]) else val[n["a3"]]
        else: r = 0.0
        val[ni] = r
    return val[root]

def eval_closure(closure, pos, slot, peak, root, corner, iy):
    """闭包-slot 求值：按闭包位置升序，val[slot]。返回根值。"""
    val = [0.0] * peak
    for ci, ni in enumerate(closure):
        n = nodes[ni]; t = n["type"]
        def ca(f):
            v = n[f]
            return pos[v] if (v >= 0 and v in pos and f in read_fields.get(t, ())) else v
        a1, a2, a3 = ca("a1"), ca("a2"), ca("a3")
        f0, f1, f2, f3 = n["f0"], n["f1"], n["f2"], n["f3"]
        if t == 0: r = f0
        elif t == 1: r = float(iy)
        elif t == 18: r = 0.0 if f1 == f0 else f2 + max(0.0, min(1.0, (iy - f0) / (f1 - f0))) * (f3 - f2)
        elif t in (2, 19): r = m_noise(a1, corner)
        elif t == 3: r = m_old(a1, corner)
        elif t == 4: r = m_spline(a1, corner)
        elif t == 22:
            v = val[slot[ca('a1')]]; r = ws_scale(int(f0), v) * abs(m_noise(a2, corner))
        elif t in (20, 21): r = val[slot[ca('a1')]]
        elif t == 6: r = val[slot[a1]] + val[slot[a2]]
        elif t == 7: r = val[slot[a1]] * val[slot[a2]]
        elif t == 8: r = min(val[slot[a1]], val[slot[a2]])
        elif t == 9: r = max(val[slot[a1]], val[slot[a2]])
        elif t == 10: r = abs(val[slot[a1]])
        elif t == 11: v = val[slot[a1]]; r = v * v
        elif t == 12: v = val[slot[a1]]; r = v * v * v
        elif t == 13: v = val[slot[a1]]; r = v if v > 0 else v * 0.5
        elif t == 14: v = val[slot[a1]]; r = v if v > 0 else v * 0.25
        elif t == 15:
            v = val[slot[a1]]; c = 1.0 if v > 1 else (-1.0 if v < -1 else v); r = c / 2 - c * c * c / 24
        elif t == 16:
            v = val[slot[a1]]; r = f1 if v > f1 else (f0 if v < f0 else v)
        elif t == 17:
            inp = val[slot[a1]]; r = val[slot[a2]] if (f0 <= inp < f1) else val[slot[a3]]
        else: r = 0.0
        val[slot[ci]] = r
    return val[slot[pos[root]]]

fail = 0
# --- interp closures ---
for k, (closure, pos, slot, peak) in enumerate(closures):
    root = g.interp_roots[k]
    subset = {ni: ci for ci, ni in enumerate(closure)}
    for corner in range(8):
        for iy in (-64, -10, 0, 40, 90):
            rv_ref = eval_ref(subset, root, corner, iy)
            rv_cl = eval_closure(closure, pos, slot, peak, root, corner, iy)
            if abs(rv_ref - rv_cl) > 1e-9:
                fail += 1
                print(f"  !! interp{k} root={root} corner={corner} iy={iy}: ref={rv_ref} closure={rv_cl}")
print("interp closure sim (root value identical):", "OK" if fail == 0 else f"FAIL ({fail})")

# --- top closure (DF_INTERP mocked via eval_closure for leaf interp) ---
# top closure has DF_INTERP nodes; mock interp_N(a1)=m_spline(a1*7, 0) to keep deterministic distinct.
def eval_closure_top(corner, iy):
    closure, pos, slot, peak = top_closure, top_pos, top_slot, layout["top_peak"]
    val = [0.0] * peak
    for ci, ni in enumerate(closure):
        n = nodes[ni]; t = n["type"]
        def ca(f):
            v = n[f]
            return pos[v] if (v >= 0 and v in pos and f in read_fields.get(t, ())) else v
        a1, a2, a3 = ca("a1"), ca("a2"), ca("a3")
        f0, f1, f2, f3 = n["f0"], n["f1"], n["f2"], n["f3"]
        if t == 5: r = m_spline(a1 * 7, 0)   # mock interp_N 返回值 = f(a1=interp index)
        elif t == 0: r = f0
        elif t == 1: r = float(iy)
        elif t == 18: r = 0.0 if f1 == f0 else f2 + max(0.0, min(1.0, (iy - f0) / (f1 - f0))) * (f3 - f2)
        elif t in (2, 19): r = m_noise(a1, 0)
        elif t == 3: r = m_old(a1, 0)
        elif t == 4: r = m_spline(a1, 0)
        elif t == 22:
            v = val[slot[ca('a1')]]; r = ws_scale(int(f0), v) * abs(m_noise(a2, 0))
        elif t in (20, 21): r = val[slot[ca('a1')]]
        elif t == 6: r = val[slot[a1]] + val[slot[a2]]
        elif t == 7: r = val[slot[a1]] * val[slot[a2]]
        elif t == 8: r = min(val[slot[a1]], val[slot[a2]])
        elif t == 9: r = max(val[slot[a1]], val[slot[a2]])
        elif t == 10: r = abs(val[slot[a1]])
        elif t == 11: v = val[slot[a1]]; r = v * v
        elif t == 12: v = val[slot[a1]]; r = v * v * v
        elif t == 13: v = val[slot[a1]]; r = v if v > 0 else v * 0.5
        elif t == 14: v = val[slot[a1]]; r = v if v > 0 else v * 0.25
        elif t == 15:
            v = val[slot[a1]]; c = 1.0 if v > 1 else (-1.0 if v < -1 else v); r = c / 2 - c * c * c / 24
        elif t == 16:
            v = val[slot[a1]]; r = f1 if v > f1 else (f0 if v < f0 else v)
        elif t == 17:
            inp = val[slot[a1]]; r = val[slot[a2]] if (f0 <= inp < f1) else val[slot[a3]]
        else: r = 0.0
        val[slot[ci]] = r
    return val[slot[pos[n_nodes - 1]]]

# 顶层理想参考：可达子树 + DF_INTERP mock
def eval_ref_top(corner, iy):
    subset = {ni: ci for ci, ni in enumerate(top_closure)}
    val = [0.0] * n_nodes
    for ni in sorted(subset.keys()):
        n = nodes[ni]; t = n["type"]
        if t == 5: r = m_spline(n["a1"] * 7, 0)
        elif t == 0: r = n["f0"]
        elif t == 1: r = float(iy)
        elif t == 18: r = 0.0 if n["f1"] == n["f0"] else n["f2"] + max(0.0, min(1.0, (iy - n["f0"]) / (n["f1"] - n["f0"]))) * (n["f3"] - n["f2"])
        elif t in (2, 19): r = m_noise(n["a1"], 0)
        elif t == 3: r = m_old(n["a1"], 0)
        elif t == 4: r = m_spline(n["a1"], 0)
        elif t == 22:
            v = val[n["a1"]]; r = ws_scale(int(n["f0"]), v) * abs(m_noise(n["a2"], 0))
        elif t in (20, 21): r = val[n["a1"]]
        elif t == 6: r = val[n["a1"]] + val[n["a2"]]
        elif t == 7: r = val[n["a1"]] * val[n["a2"]]
        elif t == 8: r = min(val[n["a1"]], val[n["a2"]])
        elif t == 9: r = max(val[n["a1"]], val[n["a2"]])
        elif t == 10: r = abs(val[n["a1"]])
        elif t == 11: v = val[n["a1"]]; r = v * v
        elif t == 12: v = val[n["a1"]]; r = v * v * v
        elif t == 13: v = val[n["a1"]]; r = v if v > 0 else v * 0.5
        elif t == 14: v = val[n["a1"]]; r = v if v > 0 else v * 0.25
        elif t == 15:
            v = val[n["a1"]]; c = 1.0 if v > 1 else (-1.0 if v < -1 else v); r = c / 2 - c * c * c / 24
        elif t == 16:
            v = val[n["a1"]]; r = n["f1"] if v > n["f1"] else (n["f0"] if v < n["f0"] else v)
        elif t == 17:
            inp = val[n["a1"]]; r = val[n["a2"]] if (n["f0"] <= inp < n["f1"]) else val[n["a3"]]
        else: r = 0.0
        val[ni] = r
    return val[n_nodes - 1]

tfail = 0
for iy in (-64, -10, 0, 40, 90):
    rv_ref = eval_ref_top(0, iy)
    rv_cl = eval_closure_top(0, iy)
    if abs(rv_ref - rv_cl) > 1e-9:
        tfail += 1
        print(f"  !! TOP iy={iy}: ref={rv_ref} closure={rv_cl}")
print("top closure sim (root value identical):", "OK" if tfail == 0 else f"FAIL ({tfail})")

# ALSO: verify no read of an unwritten slot in the closure (liveness ordering)
print("\n=== slot-write-before-read check (liveness safety) ===")
slot_ok = True
for k, (closure, pos, slot, peak) in enumerate(closures):
    written = set()
    for ci, ni in enumerate(closure):
        n = nodes[ni]; t = n["type"]
        def ca(f):
            v = n[f]
            return pos[v] if (v >= 0 and v in pos and f in read_fields.get(t, ())) else v
        # which slots this node reads
        rd = []
        if t in (10, 11, 12, 13, 14, 15, 16, 20, 21): rd = [slot[ca('a1')]]
        elif t == 17: rd = [slot[ca('a1')], slot[ca('a2')], slot[ca('a3')]]
        elif t == 22: rd = [slot[ca('a1')]]
        elif t in (6, 7, 8, 9): rd = [slot[ca('a1')], slot[ca('a2')]]
        for s in rd:
            if s not in written:
                slot_ok = False
                print(f"  !! interp{k} ci={ci} node={ni} reads slot {s} not yet written")
        written.add(slot[ci])
print("interp slot-write-before-read:", "OK" if slot_ok else "FAIL")

if fail == 0 and tfail == 0 and slot_ok:
    print("\n===== SIM RESULT: OK (closure == reachable subtree, bit-identical) =====")
else:
    print("\n===== SIM RESULT: FAIL =====")
    sys.exit(1)
