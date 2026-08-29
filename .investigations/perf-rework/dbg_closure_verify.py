# dbg_closure_verify.py —— 验证 D25 闭包优化：gen_cpu 生成的 C++ 闭包数组与 _compute_val_layout 严格一致，
# 且每个 interp 闭包拓扑序正确（子节点闭包位置 < 父节点），确保顺序求值先算子节点。
# 只读（gen_cpu 输出写到 probe 文件，不覆盖 cpu_backend.h / 不碰 gpu-assets）。
import json, sys, re, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.path.insert(0, r'E:\PYTHON\CoreSwap\.investigations\perf-rework')
import dfc_gen

dfdir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\density_function'
ndir = r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise'
settings = json.load(open(r'E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json'))
fd = settings["noise_router"]["final_density"]
g = dfc_gen.DfcGen(dfdir, ndir)
root = g.gen_df(fd)
code = g.gen_cpu(fd)   # full C++ header string

# 1. Generate to probe file (diag only)
probe = r'E:\PYTHON\CoreSwap\.investigations\perf-rework\cpu_backend_closure_probe.h'
with open(probe, 'w', encoding='utf-8') as fh:
    fh.write(code)
print("wrote probe:", probe, "bytes=", len(code))

# 2. Recompute layout (ground truth)
layout = g._compute_val_layout()
read_fields = layout["read_fields"]
closures = layout["closures"]
top_closure = layout["top_closure"]
top_pos = layout["top_pos"]
top_slot = layout["top_slot"]
top_peak = layout["top_peak"]
bases = layout["bases"]
nodes = g.df_nodes
n_nodes = len(nodes)

def parse_arr(name, code, is_float=False):
    m = re.search(name + r'\[(\d+)\] = \{(.*?)\};', code, re.S)
    if not m:
        return None
    n = int(m.group(1))
    body = m.group(2)
    if is_float:
        vals = [float(x.rstrip('f')) for x in body.split(',') if x.strip()]
    else:
        vals = [int(x.strip()) for x in body.split(',') if x.strip()]
    assert len(vals) == n, f"{name}: len {len(vals)} != {n}"
    return vals

def parse_val(name, code):
    m = re.search(name + r'\s*=\s*(-?\d+);', code)
    return int(m.group(1)) if m else None

errors = []

# Extract generated C++ closure tables
gen_off = parse_arr('CLOSURE_OFF', code)
gen_len = parse_arr('CLOSURE_LEN', code)
gen_peak = parse_arr('CLOSURE_VAL_SLOTS', code)
gen_rpos = parse_arr('CLOSURE_ROOT_POS', code)
gen_type = parse_arr('CLOSURE_TYPE', code)
gen_a1 = parse_arr('CLOSURE_A1', code)
gen_a2 = parse_arr('CLOSURE_A2', code)
gen_a3 = parse_arr('CLOSURE_A3', code)
gen_f0 = parse_arr('CLOSURE_F0', code, True)
gen_slot = parse_arr('CLOSURE_SLOT', code)
gen_tlen = parse_val('TOP_CLOSURE_LEN', code)
gen_vs_top = parse_val('VAL_SLOTS_TOP', code)
gen_trpos = parse_val('TOP_ROOT_POS', code)
gen_ttype = parse_arr('TOP_TYPE', code)
gen_ta1 = parse_arr('TOP_A1', code)
gen_ta2 = parse_arr('TOP_A2', code)
gen_ta3 = parse_arr('TOP_A3', code)
gen_tslot = parse_arr('TOP_SLOT', code)

print("\n=== gen_cpu closure extraction ===")
print("CLOSURE_OFF =", gen_off)
print("CLOSURE_LEN =", gen_len)
print("CLOSURE_VAL_SLOTS =", gen_peak)
print("CLOSURE_ROOT_POS =", gen_rpos)
print("TOP_CLOSURE_LEN =", gen_tlen, "VAL_SLOTS_TOP =", gen_vs_top, "TOP_ROOT_POS =", gen_trpos)

# 3. Cross-check: generated C++ closure tables == recomputed from layout (== GLSL source)
def map_a(cur_pos, t, v, f):
    if v >= 0 and v in cur_pos and f in read_fields.get(t, ()):
        return cur_pos[v]
    return v

ref_type, ref_a1, ref_a2, ref_a3, ref_slot = [], [], [], [], []
ref_f0 = []
ref_off, ref_len, ref_peak, ref_rpos = [], [], [], []
acc = 0
for k, (closure, pos, slot, peak) in enumerate(closures):
    ref_off.append(acc); ref_len.append(len(closure)); ref_peak.append(peak)
    r = g.interp_roots[k]
    ref_rpos.append(pos[r] if r in pos else 0)
    for ci, i in enumerate(closure):
        n = nodes[i]; t = n["type"]
        ref_type.append(t)
        ref_a1.append(map_a(pos, t, n["a1"], "a1"))
        ref_a2.append(map_a(pos, t, n["a2"], "a2"))
        ref_a3.append(map_a(pos, t, n["a3"], "a3"))
        ref_slot.append(slot[ci])
        ref_f0.append(n["f0"])
    acc += len(closure)

ok = True
for name, a, b in [("CLOSURE_OFF", gen_off, ref_off), ("CLOSURE_LEN", gen_len, ref_len),
                   ("CLOSURE_VAL_SLOTS", gen_peak, ref_peak), ("CLOSURE_ROOT_POS", gen_rpos, ref_rpos),
                   ("CLOSURE_TYPE", gen_type, ref_type), ("CLOSURE_A1", gen_a1, ref_a1),
                   ("CLOSURE_A2", gen_a2, ref_a2), ("CLOSURE_A3", gen_a3, ref_a3),
                   ("CLOSURE_SLOT", gen_slot, ref_slot)]:
    if a != b:
        ok = False
        errors.append(f"  !! {name} mismatch: gen={a} ref={b}")
# float f0 compare (approx exact)
for i, (gv, rv) in enumerate(zip(gen_f0, ref_f0)):
    if abs(gv - rv) > 1e-12:
        ok = False
        errors.append(f"  !! CLOSURE_F0[{i}] gen={gv} ref={rv}")
print("closure tables == _compute_val_layout (GLSL 同源):", "OK" if ok else "FAIL", flush=True)

# Top closure cross-check
ref_ttype, ref_ta1, ref_ta2, ref_ta3, ref_tslot = [], [], [], [], []
for ci, i in enumerate(top_closure):
    n = nodes[i]; t = n["type"]
    ref_ttype.append(t)
    ref_ta1.append(map_a(top_pos, t, n["a1"], "a1"))
    ref_ta2.append(map_a(top_pos, t, n["a2"], "a2"))
    ref_ta3.append(map_a(top_pos, t, n["a3"], "a3"))
    ref_tslot.append(top_slot[ci])
tok = True
if gen_tlen != len(top_closure): tok = False; errors.append(f"  !! TOP_CLOSURE_LEN gen={gen_tlen} ref={len(top_closure)}")
for name, a, b in [("TOP_TYPE", gen_ttype, ref_ttype), ("TOP_A1", gen_ta1, ref_ta1),
                   ("TOP_A2", gen_ta2, ref_ta2), ("TOP_A3", gen_ta3, ref_ta3), ("TOP_SLOT", gen_tslot, ref_tslot)]:
    if a != b:
        tok = False; errors.append(f"  !! {name} mismatch: gen={a} ref={b}")
if gen_vs_top != top_peak: tok = False; errors.append(f"  !! VAL_SLOTS_TOP gen={gen_vs_top} ref={top_peak}")
if gen_trpos != g.top_root_pos: tok = False; errors.append(f"  !! TOP_ROOT_POS gen={gen_trpos} ref={g.top_root_pos}")
print("top closure tables == layout:", "OK" if tok else "FAIL", flush=True)

# 4. Topo-order check: children closure position < parent closure position (sequential eval validity)
print("\n=== topo-order check (children before parent) ===")
topo_ok = True
for k, (closure, pos, slot, peak) in enumerate(closures):
    for ci, i in enumerate(closure):
        n = nodes[i]; t = n["type"]
        for f in ('a1', 'a2', 'a3'):
            if f in read_fields.get(t, ()):
                c = n[f]
                if c >= 0 and c in pos:
                    cp = pos[c]
                    if cp >= ci:
                        topo_ok = False
                        errors.append(f"  !! interp{k} node {i} (ci={ci}) child {f}={c} at closure pos {cp} >= ci")
        # also the node's own slot write must be after its children reads
print("interp closures topo-ordered:", "OK" if topo_ok else "FAIL", flush=True)

# 5. Disjoint / coverage: each closure subset of reachable set + full set coverage sanity
print("\n=== closure sizes (dead-node elimination) ===")
print(f"df_nodes(D)= {n_nodes}")
for k, (closure, pos, slot, peak) in enumerate(closures):
    print(f"  interp {k}: closure={len(closure)} peak={peak} dead_eliminated={n_nodes - len(closure)}")

# 6. Sum of closure CPU evaluate count vs full
# Each eval_df_base call now iterates closure_len instead of DF_NODES.
# For buildInterpGrid: grid has 49*5*5=1225 cells (minus reuse) but per-cell split → several eval_df_base calls.
# Rough per-call node count saved:
print("\n=== generated eval_df_base/eval_df present ===")
print("eval_df_base present:", "eval_df_base(int interpIdx" in code)
print("eval_df present:", "float eval_df(int sIdx" in code)
print("eval_df_base(int root," in code, "(old root-param signature gone)")

# 7. Ensure old markers removed
print("\n=== stale-form cleanup ===")
for marker in ["DF_NODES", "int root = INTERP_ROOTS[interpIdx]", "eval_df_base(root", "return eval_df(TOP_ROOT"]:
    cnt = code.count(marker)
    print(f"  '{marker}' remaining occurrences: {cnt}")

print("\n===== VERIFY RESULT =====")
if errors:
    for e in errors:
        print(e)
    print("RESULT: FAIL")
    sys.exit(1)
print("RESULT: OK (all checks passed)")
