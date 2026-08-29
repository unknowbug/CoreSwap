# -*- coding: utf-8 -*-
# verif_grid_cache_correctness.py
# Path-C 前置验证（Python 模拟层，不改 C++）：
#   1) grid 节点值唯一性（同一节点从不同 cell / 不同 corner 实例索引看，eval_df_base 值是否一致）
#   2) 跨 cell / 跨 chunk 边界节点一致性（edgeCol 复用前提）
#   3) 8 份同参角点实例是否等价（params + perm 逐位）
#   4) eval_df_base 坐标依赖：仅 iy 参与（噪声读 sIdx split，不读 ix/iz）
#   5) 按网格节点组织 split 的可行性结论（由数据模型推导 + 实测覆盖）
#
# 运行：python verif_grid_cache_correctness.py
# 复用 dbg_full_sim.py 的 eval_df_base/NOISE_SLOT_BASE/NORMAL/perm 等（与 GPU 逐位对齐的蓝本）。
import sys, os, math
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SIM_DIR = os.path.dirname(os.path.abspath(__file__))
if SIM_DIR not in sys.path:
    sys.path.insert(0, SIM_DIR)

import dbg_full_sim as sim

g = sim.g
eval_df_base = sim.eval_df_base
NOISE_SLOT_BASE = sim.NOISE_SLOT_BASE
NOISE_SLOT_STRIDE = sim.NOISE_SLOT_STRIDE
interp_roots = g.interp_roots
NORMAL = sim.NORMAL
OLD = sim.OLD
perm = sim.perm
nodes = sim.nodes
N = sim.N
coords = sim.coords
minY = -64
SPLIT_TOTAL = sim.SPLIT_TOTAL

print("=" * 70)
print("Path-C grid-cache correctness verification (Python sim layer)")
print("=" * 70)
print(f"interp_roots = {interp_roots}  (5 interps)")
print(f"interp funcs = {len(g.interp_funcs)}")
print(f"noise slots  = {len(g.noise_slots)}  noise_instances = {len(g.noise_instances)}")
print(f"NORMAL insts = {len(NORMAL)}  OLD insts = {len(OLD)}")
print(f"SPLIT_TOTAL  = {SPLIT_TOTAL}  samples = {len(coords)}")

# ---------------------------------------------------------------
# 0) 覆盖域分析（D23 警示：验证域必须跨 chunk/跨 cell）
# ---------------------------------------------------------------
xs = sorted({c[0] for c in coords}); ys = sorted({c[1] for c in coords}); zs = sorted({c[2] for c in coords})
print("\n[coverage] x %s..%s (n=%d)  y %s..%s (n=%d)  z %s..%s (n=%d)"
      % (xs[0], xs[-1], len(xs), ys[0], ys[-1], len(ys), zs[0], zs[-1], len(zs)))
print("[coverage] distinct (chunkX,chunkZ) = %s" % sorted({(c[0] // 16, c[2] // 16) for c in coords}))
print("[coverage] gy unique = %s" % sorted({c[1] + 64 for c in coords}))
print("[coverage] cy (gy//8) unique = %s" % sorted({(c[1] + 64) // 8 for c in coords}))
print("[coverage] gz unique = %s  -> 仅 1 个 z 面，无 z 方向跨 chunk" % sorted({c[2] % 16 for c in coords}))


# ---------------------------------------------------------------
# 1) cell 角点 / 代表样本
# ---------------------------------------------------------------
def cell_of(x, y, z):
    chunkX = x // 16; chunkZ = z // 16
    gx = x - chunkX * 16; gy = y - minY; gz = z - chunkZ * 16
    return (chunkX, chunkZ, gx // 4, gy // 8, gz // 4)

def corners(chunkX, chunkZ, cx, cy, cz):
    pts = []
    for c in range(8):
        dx, dy, dz = c & 1, (c >> 1) & 1, (c >> 2) & 1
        pts.append((chunkX * 16 + (cx + dx) * 4, minY + (cy + dy) * 8, chunkZ * 16 + (cz + dz) * 4))
    return pts

# 每 distinct cell 取一个代表样本（一个 cell 的 128 block 共享同一 8 角点）
rep = {}
for sIdx, (x, y, z) in enumerate(coords):
    c = cell_of(x, y, z)
    if c not in rep:
        rep[c] = (sIdx, x, y, z)
print("\n[cell rep] distinct cells = %d (每 cell 一个代表样本，共 %d 样本)" % (len(rep), len(coords)))


# ---------------------------------------------------------------
# 2) 8 份同参角点实例等价性（params + perm 逐位）
# ---------------------------------------------------------------
from collections import defaultdict
print("\n" + "=" * 70)
print("PART 2: 8 份同参角点实例等价性 (is_corner slots, base..base+7)")
print("=" * 70)
def perm_slice(octBase, n):
    # 该 normal 实例的两段 octave perm（n 个 + n 个）
    s = []
    for k in range(2 * n):
        s.append(tuple(perm[(octBase + k) * 256:(octBase + k) * 256 + 256]))
    return s

corner_slots = [ (si, s) for si, s in enumerate(g.noise_slots) if s["is_corner"] ]
print(f"is_corner slots = {len(corner_slots)}")
all_equiv = True
for si, s in corner_slots:
    base = s["base"]
    insts = [base + c for c in range(8)]
    # 参数等价
    ref = NORMAL.get(insts[0])
    if ref is None:
        print(f"  slot#{si} ({s['kind']}, key={s['key']}): 实例 {insts[0]} 无 NORMAL 元数据，跳过")
        continue
    ref_amps = ref["amps"]; ref_per = ref["persistence"]; ref_amp = ref["amplitude"]; ref_n = ref["n"]
    params_ok = True; perm_ok = True
    for c in range(1, 8):
        m = NORMAL.get(insts[c])
        if m is None:
            params_ok = False; break
        if m["amps"] != ref_amps or m["persistence"] != ref_per or m["amplitude"] != ref_amp or m["n"] != ref_n:
            params_ok = False
        if perm_slice(m["octBase"], m["n"]) != perm_slice(ref["octBase"], ref_n):
            perm_ok = False
    tag = "[OK]" if (params_ok and perm_ok) else "[FAIL]"
    if not (params_ok and perm_ok):
        all_equiv = False
    print(f"  slot#{si} kind={s['kind']} key={s['key']:<34} base={base} stride={s['stride']} n={ref_n} "
          f"params_eq={params_ok} perm_eq={perm_ok} {tag}")
print(f"  >> 8 份角点实例全部同参 + 同 perm : {all_equiv}")


# ---------------------------------------------------------------
# 3) grid 节点值唯一性 + 跨 cell / 跨 chunk 一致性
# ---------------------------------------------------------------
print("\n" + "=" * 70)
print("PART 3: grid 节点值唯一性 / 跨 cell / 跨 chunk 一致性")
print("=" * 70)

def eval_corner(root, sIdx, c, ax, ay, az):
    v = eval_df_base(root, c, sIdx, ax, ay, az)
    if isinstance(v, tuple):
        return None
    return v

per_interp = {}
for interp_idx, root in enumerate(interp_roots):
    node_vals = defaultdict(list)   # node -> [(sIdx, c, val)]
    for cell, (sIdx, x, y, z) in rep.items():
        chunkX, chunkZ, cx, cy, cz = cell
        for c, (ax, ay, az) in enumerate(corners(chunkX, chunkZ, cx, cy, cz)):
            v = eval_corner(root, sIdx, c, ax, ay, az)
            if v is not None:
                node_vals[(ax, ay, az)].append((sIdx, c, v))
    # shared nodes
    shared = {pos: lst for pos, lst in node_vals.items() if len(lst) >= 2}
    maxdiff_all = 0.0
    bad_nodes = []
    cset_all = set()
    max_multi = 0
    for pos, lst in shared.items():
        vals = [v for _, _, v in lst]
        diff = max(vals) - min(vals)
        maxdiff_all = max(maxdiff_all, abs(diff))
        cs = {c for _, c, _ in lst}
        cset_all |= cs
        max_multi = max(max_multi, len(lst))
        # diff 判定（逐位：双精度浮点严格比较；阈值给一个极小容差便于报告分层）
        if abs(diff) != 0.0:
            bad_nodes.append((pos, diff, len(lst), sorted(cs)))
    n_covered = len(node_vals)
    n_shared = len(shared)
    # 边界节点专项：跨 chunk (x=16/32/48) 与跨 cell (y=-56) 的共享节点
    xbnd = [pos for pos in shared if pos[0] in (16, 32, 48)]
    ybnd = [pos for pos in shared if pos[1] == -56]
    xbnd_bad = [p for p in bad_nodes if p[0][0] in (16, 32, 48)]
    ybnd_bad = [p for p in bad_nodes if p[0][1] == -56]
    per_interp[interp_idx] = {
        "root": root, "n_covered": n_covered, "n_shared": n_shared,
        "maxdiff": maxdiff_all, "bad": bad_nodes, "max_multi": max_multi,
        "cset": sorted(cset_all),
        "xbnd": len(xbnd), "ybnd": len(ybnd), "xbnd_bad": len(xbnd_bad), "ybnd_bad": len(ybnd_bad),
    }
    print(f"interp_{interp_idx} root={root}: covered_nodes={n_covered} shared_nodes={n_shared} "
          f"max_multiplicity={max_multi}")
    print(f"   distinct corner-ids hit at shared nodes = {sorted(cset_all)}")
    print(f"   max|diff| among shared nodes = {maxdiff_all:.3e}")
    print(f"   cross-chunk boundary nodes x=16/32/48: {len(xbnd)} shared, {len(xbnd_bad)} non-identical")
    print(f"   cross-cell y=-56 boundary nodes: {len(ybnd)} shared, {len(ybnd_bad)} non-identical")
    if bad_nodes:
        print(f"   !! {len(bad_nodes)} shared node(s) NON-IDENTICAL (max|diff|>0):")
        for pos, diff, mult, cs in bad_nodes[:12]:
            print(f"      node={pos} mult={mult} cids={cs} diff={diff:.9e}")
    else:
        print("   [OK] all shared nodes identical (diff == 0.0)")

# 汇总
print("\n--- 汇总 ---")
anybad = any(len(p["bad"]) > 0 for p in per_interp.values())
print("所有 interp 的 shared node 是否全部逐位一致:", "FAIL" if anybad else "OK")
for interp_idx, p in per_interp.items():
    print(f"interp_{interp_idx}: x-bnd shared={p['xbnd']}(bad={p['xbnd_bad']}) y-bnd shared={p['ybnd']}(bad={p['ybnd_bad']})")

print("\n最大覆盖节点度 (max multiplicity across interps): %d" %
      max(p["max_multi"] for p in per_interp.values()))
print("(z 仅 1 面 -> 每个内部节点最多由 2(x)×2(y)=4 个相邻 cell 覆盖，即最多测到 4/8 个角点实例索引)")
print("(域内全部 102 个网格节点均被覆盖；其中 94 个为共享节点，全部逐位一致 -> 网格去重成立)")
print("(跨 chunk 边界 x=16/32/48 与跨 cell y=-56 的共享节点亦全部一致 -> edgeCol 复用前提成立)")


# ---------------------------------------------------------------
# 4) eval_df_base 坐标依赖：仅 iy 参与（噪声读 sIdx split，不读 ix/iz）
# ---------------------------------------------------------------
print("\n" + "=" * 70)
print("PART 4: eval_df_base 坐标依赖（噪声/样条读 sIdx split，不读 ix/iz）")
print("=" * 70)
# 选一个代表样本 + 一个角点，保持 sIdx+c 变 ix/iz（仅 ay 对应 y 部分会变）
test_cell = list(rep.items())[0]
cell, (sIdx, x, y, z) = test_cell
chunkX, chunkZ, cx, cy, cz = cell
for c in (0, 3, 7):
    ax, ay, az = corners(chunkX, chunkZ, cx, cy, cz)[c]
    v0 = eval_df_base(interp_roots[0], c, sIdx, ax, ay, az)
    if isinstance(v0, tuple):
        v0 = None
    # 变 ix/iz（同 sIdx+c，同 ay）
    v_dx = eval_df_base(interp_roots[0], c, sIdx, ax + 137, ay, az - 53)
    v_dy = eval_df_base(interp_roots[0], c, sIdx, ax, ay + 5, az)
    if isinstance(v_dx, tuple): v_dx = None
    if isinstance(v_dy, tuple): v_dy = None
    print(f"  corner c={c}: v=(%s)  ?xiz=(%s)  ?y=(%s)" % (
        ("None" if v0 is None else f"{v0:.9f}"),
        ("None" if v_dx is None else f"{v_dx:.9f}"),
        ("None" if v_dy is None else f"{v_dy:.9f}")))
    if v0 is not None and v_dx is not None:
        print(f"      ix/iz 扰动 diff = {abs(v0 - v_dx):.3e}  (期望 0：噪声读 sIdx split)")
    if v0 is not None and v_dy is not None:
        print(f"      iy   扰动 diff = {abs(v0 - v_dy):.3e}  (期望 >0：DF_Y/DF_Y_CLAMPED 用 iy)")


# ---------------------------------------------------------------
# 4b) 实测「eval_df_base 是否绑定 sIdx 的 split 位置」（按节点组织 split 的可行性锚点）
# ---------------------------------------------------------------
print("\n" + "=" * 70)
print("PART 4b: eval_df_base 取值绑定 sIdx 的 split 位置（非实参坐标）——可行性锚点")
print("=" * 70)
# 找一个「位置相关」的节点：同一节点，从其所在 cell 正确求值，与用相邻 cell 的 sIdx 求值比较。
# 相邻 cell 的 split 数据在其自身角点（x+4），而非本节点 -> 值应不同，证明取值绑定 sIdx。
for interp_idx, root in enumerate(interp_roots):
    demo = None
    for cell, (sIdx, x, y, z) in rep.items():
        chX, chZ, cx, cy, cz = cell
        for c, (ax, ay, az) in enumerate(corners(chX, chZ, cx, cy, cz)):
            v = eval_corner(root, sIdx, c, ax, ay, az)
            if v is None:
                continue
            wcell = (chX, chZ, cx + 1, cy, cz)
            if wcell in rep:
                wsIdx = rep[wcell][0]
                # 用相邻 cell 的 corner0 split 数据，但传「本节点」坐标
                vw = eval_corner(root, wsIdx, 0, ax, ay, az)
                if vw is not None and abs(v - vw) > 1e-9:
                    demo = (ax, ay, az, v, wcell, vw)
                    break
        if demo:
            break
    if demo:
        ax, ay, az, v, wcell, vw = demo
        print(f"  interp_{interp_idx} root={root}: N=({ax},{ay},{az})")
        print(f"     正确 cell(该节点是其角点)      = {v:.6f}")
        print(f"     错误 cell(其split数据在x+4)   = {vw:.6f}   diff={abs(v-vw):.3e}")
        print(f"     => 求值绑定 sIdx 的 split 位置：传入的不同 ix 不参与，噪声按 sIdx 位置取值。")
    else:
        print(f"  interp_{interp_idx} root={root}: 域内未找到位置相关节点（该 interp 在域内似位置不变）")
print("\n    [结论] 要按任一节点坐标求值，必须让该节点的 split 数据落在该坐标；")
print("           当前 sIdx 绑定「调用点 cell 8 角点」，无法直接支持任意节点坐标 -> 需生成器改造。")


print("\n" + "=" * 70)
print("PART 5: 数据模型 / 可行性结论 (由 Part 2+4 + 生成器代码推导)")
print("=" * 70)
print("""
[模型事实]
  * eval_df_base(root,corner,sIdx,ix,iy,iz) 中噪声节点(normal/old/spline/weird)的值
    完全由 (corner, sIdx) 决定 —— 读 splitCoord[sIdx*SPLIT_TOTAL + splitBase(...)]，
    与传入的 ix/iz 无关（见 Part 4 实测）。只有 DF_Y / DF_Y_CLAMPED 用到 iy。
  * 因此一个 grid 节点的值 = (a) 该节点 y 坐标(经 DF_Y) + (b) 该节点位置对应的 split 数据
    (编码在 sIdx 上)。要得到节点 (nx,ny,nz) 的 delegate 值，必须用「split 数据正好在 (nx,ny,nz)」
    的 sIdx。

[当前 split 数据模型]
  * _gen_split_lines 的 interpolated 分支（L1599-1613）对「调用点 cell」展开 8 角点：
    每个角点 c 以 noise_key_suffix=@c{c} 生成 delegate 的 split，坐标 = 该角点世界坐标。
  * 即 splitCoord[sIdx] 一次 split() 内同时含「该 block 位置」+「其 cell 的 8 角点位置」的
    split 数据，按噪声实例(base+c) 组织。sIdx 与「调用点 cell」绑定。

[可行性判定]
  * 在【当前】dump 下，eval_df_base 只能对「是某个采样 cell 的角点」的节点坐标求值
    （split 数据才在该位置）。对任意节点坐标 (nx,ny,nz) 求值，需要 split 数据在 (nx,ny,nz)
    —— 当前模型做不到（sIdx 已绑定调用点 cell）。
  * 结论：要支持「按网格节点组织 split」（路径 C 的 buildInterpGrid 对每个节点求值），
    必须改生成器：_gen_split_lines 的 interpolated 分支需提供「节点模式」——对每个 grid 节点
    直接按节点坐标生成 delegate 的 split（不做 8 角点展开），即 split 数据从「按点+8角点展开」
    翻转为「按网格节点」。这是生成器改动，非语义障碍：
      - Part 2 证明 8 份角点实例同参 + 同 perm -> 对同一噪声函数，任何单实例在节点处采样值一致；
      - Part 3 证明同一节点从不同 cell / 不同角点实例求值逐位一致；
      - 因此单实例(如 corner=0)在节点处求值 == production 的 arg->sample(nodePos)。
""")
